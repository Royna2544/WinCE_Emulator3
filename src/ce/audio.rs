use std::collections::{BTreeMap, VecDeque};

pub type MmResult = u32;

pub const MMSYSERR_NOERROR: MmResult = 0;
pub const MMSYSERR_ERROR: MmResult = 1;
pub const MMSYSERR_BADDEVICEID: MmResult = 2;
pub const MMSYSERR_INVALHANDLE: MmResult = 5;
pub const WAVERR_BADFORMAT: MmResult = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveFormat {
    pub format_tag: u16,
    pub channels: u16,
    pub samples_per_sec: u32,
    pub avg_bytes_per_sec: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveBuffer {
    pub guest_ptr: u32,
    pub len: u32,
}

#[derive(Debug, Clone)]
pub struct WaveOutDevice {
    pub id: u32,
    pub format: WaveFormat,
    pub volume: u32,
    pub state: WaveOutState,
    pending: VecDeque<WaveBuffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveOutState {
    Open,
    Playing,
    Paused,
    Reset,
    Closed,
}

#[derive(Debug, Clone)]
pub struct AudioSystem {
    next_id: u32,
    outputs: BTreeMap<u32, WaveOutDevice>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            next_id: 1,
            outputs: BTreeMap::new(),
        }
    }
}

impl AudioSystem {
    pub fn open_wave_out(&mut self, format: WaveFormat) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.outputs.insert(
            id,
            WaveOutDevice {
                id,
                format,
                volume: 0xffff_ffff,
                state: WaveOutState::Open,
                pending: VecDeque::new(),
            },
        );
        id
    }

    pub fn wave_out_get_num_devs(&self) -> u32 {
        1
    }

    pub fn wave_out_open(&mut self, format: WaveFormat) -> Result<u32, MmResult> {
        if !format.is_pcm() || format.block_align == 0 {
            return Err(WAVERR_BADFORMAT);
        }
        Ok(self.open_wave_out(format))
    }

    pub fn wave_out_write(&mut self, id: u32, buffer: WaveBuffer) -> MmResult {
        if self.write(id, buffer) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn write(&mut self, id: u32, buffer: WaveBuffer) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.push_back(buffer);
        output.state = WaveOutState::Playing;
        true
    }

    pub fn complete_next_buffer(&mut self, id: u32) -> Option<WaveBuffer> {
        let output = self.outputs.get_mut(&id)?;
        let buffer = output.pending.pop_front();
        if output.pending.is_empty() && output.state == WaveOutState::Playing {
            output.state = WaveOutState::Open;
        }
        buffer
    }

    pub fn reset(&mut self, id: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.clear();
        output.state = WaveOutState::Reset;
        true
    }

    pub fn pause(&mut self, id: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        output.state = WaveOutState::Paused;
        MMSYSERR_NOERROR
    }

    pub fn restart(&mut self, id: u32) -> MmResult {
        let Some(output) = self.outputs.get_mut(&id) else {
            return MMSYSERR_INVALHANDLE;
        };
        if !output.pending.is_empty() {
            output.state = WaveOutState::Playing;
        }
        MMSYSERR_NOERROR
    }

    pub fn wave_out_reset(&mut self, id: u32) -> MmResult {
        if self.reset(id) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn set_volume(&mut self, id: u32, volume: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.volume = volume;
        true
    }

    pub fn get_volume(&self, id: u32) -> Result<u32, MmResult> {
        self.outputs
            .get(&id)
            .map(|output| output.volume)
            .ok_or(MMSYSERR_INVALHANDLE)
    }

    pub fn wave_out_set_volume(&mut self, id: u32, volume: u32) -> MmResult {
        if self.set_volume(id, volume) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn close(&mut self, id: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.clear();
        output.state = WaveOutState::Closed;
        true
    }

    pub fn wave_out_close(&mut self, id: u32) -> MmResult {
        if self.close(id) {
            MMSYSERR_NOERROR
        } else {
            MMSYSERR_INVALHANDLE
        }
    }

    pub fn output(&self, id: u32) -> Option<&WaveOutDevice> {
        self.outputs.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_write_tracks_pending_buffer() {
        let mut audio = AudioSystem::default();
        let id = audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));

        assert!(audio.write(
            id,
            WaveBuffer {
                guest_ptr: 0x1000,
                len: 4096
            }
        ));
        assert_eq!(audio.output(id).unwrap().state, WaveOutState::Playing);
        assert_eq!(audio.complete_next_buffer(id).unwrap().guest_ptr, 0x1000);
    }
}

impl WaveFormat {
    pub fn pcm_16bit(channels: u16, samples_per_sec: u32) -> Self {
        let block_align = channels * 2;
        Self {
            format_tag: 1,
            channels,
            samples_per_sec,
            avg_bytes_per_sec: samples_per_sec * u32::from(block_align),
            block_align,
            bits_per_sample: 16,
        }
    }

    fn is_pcm(&self) -> bool {
        self.format_tag == 1
    }
}
