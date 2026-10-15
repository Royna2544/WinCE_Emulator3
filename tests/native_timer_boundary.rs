//! Offline diagnostic for a native Unicorn timer/stop boundary on Windows.
//!
//! This is not a regression test for either previously observed host AV: an
//! uncrashed sweep is only non-reproduction evidence. Keep it ignored and run
//! the exact test alone, with a stamped raw log and an external watchdog.

#![cfg(all(windows, feature = "unicorn"))]

use std::{
    cell::Cell,
    io::{self, Write},
    rc::Rc,
    time::{Duration, Instant},
};

use unicorn_engine::{
    RegisterMIPS, Unicorn,
    unicorn_const::{Arch, Mode, Prot, Query},
};

const CODE: u64 = 0x1000;
const NATIVE_TIMEOUT_US: u64 = 10_000;
const SAMPLES_PER_OFFSET: usize = 64;
const TIMER_ONLY_SAMPLES_PER_END: usize = 8;
const STOP_OFFSETS_US: [i64; 5] = [-2_000, -500, 0, 500, 2_000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    NoTimer,
    TimedStop,
    TimerOnly,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::NoTimer => "no_timer",
            Self::TimedStop => "timed_stop",
            Self::TimerOnly => "timer_only",
        }
    }

    fn timeout_us(self) -> u64 {
        match self {
            Self::NoTimer => 0,
            Self::TimedStop | Self::TimerOnly => NATIVE_TIMEOUT_US,
        }
    }
}

fn emit(line: &str) {
    let stderr = io::stderr();
    let mut locked = stderr.lock();
    writeln!(locked, "{line}").expect("write timer diagnostic telemetry");
    locked.flush().expect("flush timer diagnostic telemetry");
}

fn wait_until(started: Instant, delay: Duration) {
    let deadline = started + delay;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        if remaining > Duration::from_millis(1) {
            // Sleep only the coarse part; the final millisecond is spun so a
            // Windows scheduler tick does not quantize every stop to one side.
            std::thread::sleep(remaining - Duration::from_micros(500));
        } else {
            std::hint::spin_loop();
        }
    }
}

fn run_iteration(sequence: usize, arm: Arm, offset_us: i64) -> (u64, bool) {
    let stop_delay_us = (NATIVE_TIMEOUT_US as i64 + offset_us) as u64;
    let stop_target_us = (arm != Arm::TimerOnly).then_some(stop_delay_us);
    emit(&format!(
        "ITER_BEGIN seq={sequence} arm={} offset_us={offset_us} stop_target_us={stop_target_us:?} timeout_us={}",
        arm.label(),
        arm.timeout_us(),
    ));

    let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN)
        .expect("create a fresh native owner");
    uc.mem_map(CODE, 0x1000, Prot::ALL).expect("map loop page");
    // addiu s0,s0,1; j CODE; nop. The count argument is zero, so there is no
    // instruction-count stop competing with the timer or the one-shot hook.
    let words = [0x2610_0001_u32, 0x0800_0400, 0];
    let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    uc.mem_write(CODE, &bytes).expect("write MIPS loop");

    let hook_seen = Rc::new(Cell::new(false));
    let hook_stop_ok = Rc::new(Cell::new(None::<bool>));
    let hook_stop_elapsed_us = Rc::new(Cell::new(None::<u64>));
    let started_cell = Rc::new(Cell::new(Instant::now()));
    if arm != Arm::TimerOnly {
        let seen = Rc::clone(&hook_seen);
        let stop_ok = Rc::clone(&hook_stop_ok);
        let stop_elapsed_us = Rc::clone(&hook_stop_elapsed_us);
        let hook_started = Rc::clone(&started_cell);
        let delay = Duration::from_micros(stop_delay_us);
        uc.add_block_hook(1, 0, move |uc, address, _size| {
            if address != CODE || seen.replace(true) {
                return;
            }
            let started = hook_started.get();
            wait_until(started, delay);
            stop_elapsed_us.set(Some(started.elapsed().as_micros() as u64));
            stop_ok.set(Some(uc.emu_stop().is_ok()));
        })
        .expect("install one-shot native stop hook");
    }

    let started = Instant::now();
    started_cell.set(started);
    let result = uc.emu_start(CODE, 0, arm.timeout_us(), 0);
    let elapsed_us = started.elapsed().as_micros() as u64;
    let timeout_query = uc.query(Query::TIMEOUT);
    let count_query = uc.query(Query::COUNT_STOP);
    let pc = uc.reg_read(RegisterMIPS::PC);
    let seen = hook_seen.get();
    let stop_ok = hook_stop_ok.get();
    let stop_elapsed_us = hook_stop_elapsed_us.get();
    drop(uc); // The timer joined before this fresh native owner is closed.

    emit(&format!(
        "ITER_END seq={sequence} arm={} offset_us={offset_us} stop_target_us={stop_target_us:?} timeout_us={} elapsed_us={elapsed_us} hook_seen={seen} hook_stop_ok={stop_ok:?} hook_stop_elapsed_us={stop_elapsed_us:?} emu_result={result:?} timeout_query={timeout_query:?} count_query={count_query:?} pc={pc:?}",
        arm.label(),
        arm.timeout_us(),
    ));

    assert!(
        result.is_ok(),
        "native run failed at seq={sequence}: {result:?}"
    );
    assert_eq!(count_query, Ok(0), "count stop at seq={sequence}");
    let pc_value = pc.expect("read guest PC");
    assert_ne!(pc_value, 0, "guest PC zero at seq={sequence}");
    if seen {
        assert_eq!(stop_ok, Some(true), "hook stop failed at seq={sequence}");
    }
    if arm == Arm::NoTimer {
        assert!(
            seen,
            "timeout-zero control missed its hook at seq={sequence}"
        );
        assert_eq!(timeout_query, Ok(0), "timeout-zero control expired");
    }
    if arm == Arm::TimerOnly {
        assert_eq!(timeout_query, Ok(1), "timer-only control did not expire");
    }
    (timeout_query.expect("read timer result") as u64, seen)
}

#[test]
#[ignore = "offline native diagnostic; run alone with stamped evidence and an external watchdog"]
fn timer_boundary_fresh_owner_sweep() {
    emit(&format!(
        "SWEEP_BEGIN timeout_us={NATIVE_TIMEOUT_US} samples_per_offset={SAMPLES_PER_OFFSET} timer_only_per_end={TIMER_ONLY_SAMPLES_PER_END} offsets_us={STOP_OFFSETS_US:?}"
    ));
    let mut sequence = 0;
    for _ in 0..TIMER_ONLY_SAMPLES_PER_END {
        sequence += 1;
        run_iteration(sequence, Arm::TimerOnly, 0);
    }

    let mut timed_expired_total = 0;
    let mut timed_unexpired_total = 0;
    let mut timed_expired_with_hook = 0;
    let mut timed_unexpired_with_hook = 0;
    for offset_us in STOP_OFFSETS_US {
        let mut expired_at_offset = 0;
        let mut expired_with_hook_at_offset = 0;
        let mut unexpired_with_hook_at_offset = 0;
        for _ in 0..SAMPLES_PER_OFFSET {
            sequence += 1;
            run_iteration(sequence, Arm::NoTimer, offset_us);
            sequence += 1;
            let (timed_out, hook_seen) = run_iteration(sequence, Arm::TimedStop, offset_us);
            if timed_out == 1 {
                expired_at_offset += 1;
                if hook_seen {
                    expired_with_hook_at_offset += 1;
                }
            } else {
                if hook_seen {
                    unexpired_with_hook_at_offset += 1;
                }
            }
        }
        let unexpired_at_offset = SAMPLES_PER_OFFSET - expired_at_offset;
        timed_expired_total += expired_at_offset;
        timed_unexpired_total += unexpired_at_offset;
        timed_expired_with_hook += expired_with_hook_at_offset;
        timed_unexpired_with_hook += unexpired_with_hook_at_offset;
        emit(&format!(
            "OFFSET_SUMMARY offset_us={offset_us} timed_expired={expired_at_offset} timed_unexpired={unexpired_at_offset} expired_with_hook={expired_with_hook_at_offset} unexpired_with_hook={unexpired_with_hook_at_offset} matched_no_timer={} completed={sequence}",
            SAMPLES_PER_OFFSET,
        ));
    }

    for _ in 0..TIMER_ONLY_SAMPLES_PER_END {
        sequence += 1;
        run_iteration(sequence, Arm::TimerOnly, 0);
    }
    emit(&format!(
        "SWEEP_END completed={sequence} timed_expired={timed_expired_total} timed_unexpired={timed_unexpired_total} expired_with_hook={timed_expired_with_hook} unexpired_with_hook={timed_unexpired_with_hook} timer_only={} no_timer={} timed_stop={}",
        2 * TIMER_ONLY_SAMPLES_PER_END,
        STOP_OFFSETS_US.len() * SAMPLES_PER_OFFSET,
        STOP_OFFSETS_US.len() * SAMPLES_PER_OFFSET,
    ));
    assert!(
        timed_expired_with_hook > 0 && timed_unexpired_with_hook > 0,
        "inconclusive hook/timer overlap: retune timing; this is not an AV diagnosis"
    );
}
