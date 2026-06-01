use std::collections::{BTreeMap, VecDeque};

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

    pub fn set_volume(&mut self, id: u32, volume: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.volume = volume;
        true
    }

    pub fn close(&mut self, id: u32) -> bool {
        let Some(output) = self.outputs.get_mut(&id) else {
            return false;
        };
        output.pending.clear();
        output.state = WaveOutState::Closed;
        true
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
}
