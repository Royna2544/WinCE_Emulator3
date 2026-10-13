//! Yamaha magnetometer stream-device protocol.
//!
//! `YAS526B.dll` forwards byte 1 OR byte 3 to I2C address 0x2e/0x2f,
//! and reads byte 2 bytes without using a register address. These are command
//! and measurement transactions, not little-endian i16 register banks.
//! The dumped caller has two wire formats: the older two-address YAS526 path
//! (calibration select 0xa0, nibble-packed measurements) and the single-address
//! path (select 0xc8, big-endian measurements). The latter is also documented
//! by Yamaha's YAS529 driver. Each measurement command latches its own reply.
//!
//! This is a deterministic ideal-device model, not a factory-chip replay:
//! zero intrinsic DAC bias/trim, signed ADC-count field input, and instantaneous
//! conversion completion. The reported rough offset includes the five-count
//! encoding bias that the guest removes before programming the DAC. The
//! single-address model uses Yamaha's documented 226-count compensation step.
//! The older two-address DAC gain is not established: its registers are retained
//! but its analog compensation is unmodelled, and its ideal intrinsic offset is
//! zero. Wire saturation is retained, so out-of-range normal input still reaches
//! the guest's under/overflow checks. No factory-chip calibration is replayed.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_MFS_WRITE_REGISTERS: u32 = 0xb000_0000;
pub const IOCTL_MFS_READ_REGISTERS: u32 = 0xb000_0004;
pub const IOCTL_MFS_CONTROL_08: u32 = 0xb000_0008;
pub const IOCTL_MFS_CONTROL_0C: u32 = 0xb000_000c;
pub const IOCTL_MFS_CONTROL_10: u32 = 0xb000_0010;

const ADC_CENTER: i32 = 512;
const ADC_MAX: i32 = 0x7ff;
const TEMPERATURE_CENTER: u16 = 256;
const SINGLE_ADDRESS_COARSE_STEP: i32 = 226;
const ROUGH_ENCODING_BIAS: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireFormat {
    TwoAddress526,
    SingleAddress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadSelection {
    Measurement,
    Calibration,
}

#[derive(Debug, Clone)]
pub struct Magnetometer {
    format: WireFormat,
    read_selection: [ReadSelection; 2],
    replies: [[u8; 9]; 2],
    field: [i16; 3],
    // X, Y1, Y2 DAC values; these are not the reported rough-offset encoding.
    coarse_offsets: [u8; 3],
}

impl Magnetometer {
    pub fn new() -> Self {
        let mut sensor = Self {
            format: WireFormat::TwoAddress526,
            read_selection: [ReadSelection::Measurement; 2],
            replies: [[0; 9]; 2],
            field: [16, -16, 64],
            coarse_offsets: [0; 3],
        };
        sensor.latch_measurement(0, false);
        sensor.latch_measurement(1, false);
        sensor
    }

    /// Remote field values are signed ADC-count deltas in Cartesian XYZ.
    /// The Yamaha chip exposes X/Y1/Y2 around an unsigned ADC center instead.
    pub fn set_field(&mut self, x: i16, y: i16, z: i16) {
        self.field = [x, y, z];
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
        output_buffer_present: bool,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_MFS_WRITE_REGISTERS => self.write_registers(input),
            IOCTL_MFS_READ_REGISTERS => {
                self.read_registers(input, output_capacity, output_buffer_present)
            }
            IOCTL_MFS_CONTROL_08 | IOCTL_MFS_CONTROL_0C | IOCTL_MFS_CONTROL_10 => {
                success(Vec::new())
            }
            _ => failure(),
        }
    }

    fn write_registers(&mut self, input: &[u8]) -> DeviceIoControlResult {
        if input.len() != 4 || !valid_bank(input[0]) || !valid_selector(input[1]) {
            return failure();
        }
        let bank = input[0] as usize;
        // YAS526B.dll+0x15d8/+0x1638 sends this combined byte, not the
        // selector or command independently.
        let command = input[1] | input[3];
        match command {
            0xa0 => {
                self.format = WireFormat::TwoAddress526;
                self.read_selection[bank] = ReadSelection::Calibration;
            }
            0xc8 => {
                self.format = WireFormat::SingleAddress;
                self.read_selection[bank] = ReadSelection::Calibration;
            }
            0x80..=0x97 | 0xc0 => {
                self.read_selection[bank] = ReadSelection::Measurement;
            }
            0x00 => self.latch_measurement(bank, false),
            0x01 => self.latch_measurement(bank, true),
            0x02 if self.format == WireFormat::SingleAddress => {
                self.latch_measurement(bank, false);
            }
            0x20..=0x7f => self.set_coarse_offset(bank, command),
            // The real stream driver forwards other command bytes too.
            // Leave unsupported chip operations inert rather than inventing
            // an application-specific acknowledgement or sample.
            _ => {}
        }
        success(Vec::new())
    }

    fn set_coarse_offset(&mut self, bank: usize, command: u8) {
        let axis = match (self.format, bank, command & 0xe0) {
            (WireFormat::TwoAddress526, 0, 0x40) => Some(1),
            (WireFormat::TwoAddress526, 0, 0x60) => Some(2),
            (WireFormat::TwoAddress526, 1, 0x40) => Some(0),
            (WireFormat::SingleAddress, 0, 0x20) => Some(0),
            (WireFormat::SingleAddress, 0, 0x40) => Some(1),
            (WireFormat::SingleAddress, 0, 0x60) => Some(2),
            _ => None,
        };
        if let Some(axis) = axis {
            self.coarse_offsets[axis] = command & 0x1f;
        }
    }

    fn channel_deltas(&self) -> [i32; 3] {
        let [x, y, z] = self.field.map(i32::from);
        match self.format {
            // Target 0x4912ec..0x49132c: +X, Y1-Y2, -(Y1+Y2).
            WireFormat::TwoAddress526 => [x, (y - z) / 2, (-y - z) / 2],
            // Target 0x492280..0x4923b0/Yamaha: -X, Y2-Y1, +(Y1+Y2).
            WireFormat::SingleAddress => [-x, (z - y) / 2, (z + y) / 2],
        }
    }

    fn latch_measurement(&mut self, bank: usize, rough: bool) {
        let deltas = self.channel_deltas();
        let values: [u16; 3] = std::array::from_fn(|axis| {
            if rough {
                let rounded = match self.format {
                    WireFormat::TwoAddress526 => 0,
                    WireFormat::SingleAddress => {
                        let delta = deltas[axis];
                        (delta + delta.signum() * (SINGLE_ADDRESS_COARSE_STEP / 2))
                            / SINGLE_ADDRESS_COARSE_STEP
                    }
                };
                (ROUGH_ENCODING_BIAS + rounded).clamp(0, 255) as u16
            } else {
                let compensation = match self.format {
                    WireFormat::TwoAddress526 => 0,
                    WireFormat::SingleAddress => {
                        i32::from(self.coarse_offsets[axis]) * SINGLE_ADDRESS_COARSE_STEP
                    }
                };
                (ADC_CENTER + deltas[axis] - compensation).clamp(0, ADC_MAX) as u16
            }
        });
        let mut reply = [0; 9];
        match self.format {
            WireFormat::TwoAddress526 => {
                if bank == 0 {
                    put_nibble_word(&mut reply[0..2], values[1]);
                    put_nibble_word(&mut reply[2..4], values[2]);
                } else {
                    put_nibble_word(&mut reply[0..2], values[0]);
                }
            }
            WireFormat::SingleAddress => {
                for (index, value) in values.into_iter().rev().enumerate() {
                    reply[index * 2..index * 2 + 2].copy_from_slice(&value.to_be_bytes());
                }
                if !rough {
                    reply[6..8].copy_from_slice(&TEMPERATURE_CENTER.to_be_bytes());
                }
            }
        }
        self.replies[bank] = reply;
    }

    fn read_registers(
        &self,
        input: &[u8],
        output_capacity: u32,
        output_buffer_present: bool,
    ) -> DeviceIoControlResult {
        if input.len() != 4 || !valid_bank(input[0]) {
            return failure();
        }
        let requested = input[2] as usize;
        if requested == 0 || !output_buffer_present {
            return failure();
        }
        // The dumped driver deliberately ignores dwOutSize. Preserve the
        // observed non-null-buffer/zero-capacity path, while rejecting an
        // explicitly undersized host destination.
        if output_capacity != 0 && requested > output_capacity as usize {
            return failure();
        }
        let bank = input[0] as usize;
        let reply = match self.read_selection[bank] {
            ReadSelection::Measurement => self.replies[bank],
            ReadSelection::Calibration => self.calibration_reply(),
        };
        let output = (0..requested)
            .map(|index| reply.get(index).copied().unwrap_or(0))
            .collect();
        success(output)
    }

    fn calibration_reply(&self) -> [u8; 9] {
        match self.format {
            // Zero signed trim coefficients in the older two-bank encoding.
            // Its intrinsic diagonal coefficients are 100/50/150, as decoded
            // by target VA 0x49045c..0x490558. This is not live field data.
            WireFormat::TwoAddress526 => [0; 9],
            // Identity correction matrix and zero temperature coefficients in
            // the target VA 0x491758..0x4918f4/Yamaha calibration encoding.
            WireFormat::SingleAddress => [0x82, 0x20, 0xfa, 0x08, 0x20, 0x44, 0x80, 0x80, 0x80],
        }
    }
}

fn put_nibble_word(output: &mut [u8], value: u16) {
    output[0] = ((value >> 4) & 0x7f) as u8;
    output[1] = ((value & 0x0f) << 4) as u8;
}

fn valid_bank(value: u8) -> bool {
    matches!(value, 0 | 1)
}

fn valid_selector(value: u8) -> bool {
    matches!(value, 0x00 | 0x40 | 0x80 | 0xc0)
}

fn success(output: Vec<u8>) -> DeviceIoControlResult {
    DeviceIoControlResult {
        success: true,
        bytes_returned: output.len() as u32,
        output,
    }
}

fn failure() -> DeviceIoControlResult {
    DeviceIoControlResult {
        success: false,
        bytes_returned: 0,
        output: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(sensor: &mut Magnetometer, bank: u8, command: u8) {
        assert!(
            sensor
                .device_io_control(IOCTL_MFS_WRITE_REGISTERS, &[bank, 0, 0, command], 0, false)
                .success
        );
    }

    fn read(sensor: &mut Magnetometer, bank: u8, count: u8) -> Vec<u8> {
        sensor
            .device_io_control(IOCTL_MFS_READ_REGISTERS, &[bank, 0, count, 0], 0, true)
            .output
    }

    fn nibble_word(bytes: &[u8]) -> u16 {
        (u16::from(bytes[0] & 0x7f) << 4) | u16::from(bytes[1] >> 4)
    }

    fn single_word(bytes: &[u8]) -> u16 {
        (u16::from(bytes[0] & 7) << 8) | u16::from(bytes[1])
    }

    #[test]
    fn two_bank_rough_read_matches_dumped_offset_decoder() {
        let mut sensor = Magnetometer::new();
        for bank in 0..2 {
            write(&mut sensor, bank, 0xa0);
            assert_eq!(read(&mut sensor, bank, 4), vec![0; 4]);
            write(&mut sensor, bank, 0x80);
            write(&mut sensor, bank, 1);
        }
        let y = read(&mut sensor, 0, 4);
        let x = read(&mut sensor, 1, 2);
        // Target 0x491184..0x4911dc narrows these packed values to bytes.
        let offsets = [
            nibble_word(&x) as u8,
            nibble_word(&y[..2]) as u8,
            nibble_word(&y[2..]) as u8,
        ];
        assert_eq!(offsets, [5; 3]);
        assert!(offsets.into_iter().all(|offset| offset > 0 && offset < 33));
        let status = |offsets: [u8; 3]| {
            offsets.iter().enumerate().fold(0u8, |bits, (axis, value)| {
                bits | if *value == 0 || *value >= 33 {
                    1 << axis
                } else {
                    0
                }
            })
        };
        assert_eq!(status(offsets), 0);
        // Captured failing reads from the old fixed little-endian sample:
        // bank0 10 00 f0 ff, bank1 20 00. The target narrows to [0,0,15],
        // returning X/Y1 error bits and then initialization error -15.
        let old_offsets = [
            nibble_word(&[0x20, 0x00]) as u8,
            nibble_word(&[0x10, 0x00]) as u8,
            nibble_word(&[0xf0, 0xff]) as u8,
        ];
        assert_eq!(old_offsets, [0, 0, 15]);
        assert_eq!(status(old_offsets), 3);
    }

    #[test]
    fn two_bank_normal_read_converts_cartesian_field_and_latches() {
        let mut sensor = Magnetometer::new();
        sensor.set_field(30, -6, 42);
        write(&mut sensor, 0, 0);
        write(&mut sensor, 1, 0);
        let y = read(&mut sensor, 0, 4);
        let x = read(&mut sensor, 1, 2);
        assert_eq!(
            [nibble_word(&x), nibble_word(&y[..2]), nibble_word(&y[2..])],
            [542, 488, 494]
        );
        sensor.set_field(0, 0, 0);
        assert_eq!(read(&mut sensor, 0, 4), y);
        write(&mut sensor, 0, 0);
        assert_eq!(nibble_word(&read(&mut sensor, 0, 2)), 512);
    }

    #[test]
    fn single_address_commands_select_calibration_rough_and_normal() {
        let mut sensor = Magnetometer::new();
        write(&mut sensor, 0, 0xc8);
        let calibration = read(&mut sensor, 0, 9);
        assert_eq!(
            calibration,
            vec![0x82, 0x20, 0xfa, 0x08, 0x20, 0x44, 0x80, 0x80, 0x80]
        );
        assert_eq!(read(&mut sensor, 0, 9), calibration);
        write(&mut sensor, 0, 0xc0);
        write(&mut sensor, 0, 1);
        assert_eq!(read(&mut sensor, 0, 6), vec![0, 5, 0, 5, 0, 5]);
        sensor.set_field(30, -6, 42);
        write(&mut sensor, 0, 2);
        let data = read(&mut sensor, 0, 8);
        assert_eq!(
            [
                single_word(&data[..2]),
                single_word(&data[2..4]),
                single_word(&data[4..6]),
                single_word(&data[6..])
            ],
            [530, 536, 482, 256]
        );
    }

    #[test]
    fn two_address_forward_conversion_roundtrips_signed_field_and_quantization() {
        for (field, expected) in [
            ([30, -6, 42], [30, -6, 42]),
            ([-17, 9, -21], [-17, 9, -21]),
            ([30, -5, 42], [30, -5, 41]),
            ([-17, -8, 21], [-17, -8, 20]),
            ([0, 1, 0], [0, 0, 0]),
        ] {
            let mut sensor = Magnetometer::new();
            sensor.set_field(field[0], field[1], field[2]);
            write(&mut sensor, 0, 0);
            write(&mut sensor, 1, 0);
            let y = read(&mut sensor, 0, 4);
            let x = read(&mut sensor, 1, 2);
            let x = i32::from(nibble_word(&x)) - ADC_CENTER;
            let y1 = i32::from(nibble_word(&y[..2])) - ADC_CENTER;
            let y2 = i32::from(nibble_word(&y[2..])) - ADC_CENTER;
            // Actual B forward conversion before its correction matrix,
            // 0x4912ec..0x49132c (the target also scales each raw delta by16).
            assert_eq!([x, y1 - y2, -(y1 + y2)], expected);
        }
    }

    #[test]
    fn single_address_forward_conversion_roundtrips_signed_field_and_quantization() {
        for (field, expected) in [
            ([30, -6, 42], [30, -6, 42]),
            ([-17, 9, -21], [-17, 9, -21]),
            ([30, -5, 42], [30, -5, 41]),
            ([-17, -8, 21], [-17, -8, 20]),
            ([0, 1, 0], [0, 0, 0]),
        ] {
            let mut sensor = Magnetometer::new();
            write(&mut sensor, 0, 0xc8);
            write(&mut sensor, 0, 0xc0);
            sensor.set_field(field[0], field[1], field[2]);
            write(&mut sensor, 0, 2);
            let data = read(&mut sensor, 0, 8);
            let x = i32::from(single_word(&data[4..6])) - ADC_CENTER;
            let y1 = i32::from(single_word(&data[2..4])) - ADC_CENTER;
            let y2 = i32::from(single_word(&data[..2])) - ADC_CENTER;
            // Actual C/Yamaha forward conversion before correction matrix.
            assert_eq!([-x, y2 - y1, y1 + y2], expected);
        }
    }

    #[test]
    fn single_address_calibration_decodes_to_identity_and_zero_temperature_trim() {
        let mut sensor = Magnetometer::new();
        write(&mut sensor, 0, 0xc8);
        let calibration = read(&mut sensor, 0, 9);
        let bytes: Vec<i16> = calibration.iter().copied().map(i16::from).collect();
        // Packed OTP coefficient decoder, target 0x491758..0x491898.
        let matrix = [
            100,
            (bytes[0] >> 2) - 32,
            (((bytes[0] & 3) << 2) | (bytes[1] >> 6)) - 8,
            (bytes[1] & 0x3f) - 32,
            (bytes[2] >> 2) + 38,
            (((bytes[2] & 3) << 4) | (bytes[3] >> 4)) - 32,
            (((bytes[3] & 0xf) << 2) | (bytes[4] >> 6)) - 32,
            (bytes[4] & 0x3f) - 32,
            (bytes[5] >> 1) + 66,
        ];
        assert_eq!(matrix, [100, 0, 0, 0, 100, 0, 0, 0, 100]);
        assert_eq!(
            bytes[6..9]
                .iter()
                .map(|value| value - 128)
                .collect::<Vec<_>>(),
            vec![0; 3]
        );
    }

    #[test]
    fn selector_is_combined_with_command_and_programs_only_selected_dac() {
        let mut sensor = Magnetometer::new();
        sensor.set_field(0, 0, 0);
        assert!(
            sensor
                .device_io_control(IOCTL_MFS_WRITE_REGISTERS, &[0, 0x40, 0, 1], 0, false)
                .success
        );
        write(&mut sensor, 0, 0);
        write(&mut sensor, 1, 0);
        let y = read(&mut sensor, 0, 4);
        assert_eq!(sensor.coarse_offsets, [0, 1, 0]);
        assert_eq!([nibble_word(&y[..2]), nibble_word(&y[2..])], [512, 512]);
        assert_eq!(nibble_word(&read(&mut sensor, 1, 2)), 512);
        write(&mut sensor, 1, 0x41);
        assert_eq!(sensor.coarse_offsets, [1, 1, 0]);
        write(&mut sensor, 1, 0);
        assert_eq!(nibble_word(&read(&mut sensor, 1, 2)), 512);
        write(&mut sensor, 0, 0xc8);
        write(&mut sensor, 0, 0xc0);
        write(&mut sensor, 0, 0x22);
        write(&mut sensor, 0, 2);
        let data = read(&mut sensor, 0, 8);
        assert_eq!(single_word(&data[4..6]), 60);
    }

    #[test]
    fn wire_saturation_does_not_hide_sensor_range_errors() {
        let mut sensor = Magnetometer::new();
        sensor.set_field(i16::MIN, i16::MAX, i16::MIN);
        write(&mut sensor, 0, 0);
        write(&mut sensor, 1, 0);
        assert_eq!(nibble_word(&read(&mut sensor, 1, 2)), 0);
        assert_eq!(nibble_word(&read(&mut sensor, 0, 2)), 2047);
        write(&mut sensor, 0, 0xc8);
        write(&mut sensor, 0, 0xc0);
        write(&mut sensor, 0, 1);
        let rough = read(&mut sensor, 0, 6);
        assert_eq!(single_word(&rough[2..4]), 0);
        assert!(single_word(&rough[4..6]) >= 33);
    }

    #[test]
    fn ioctl_validation_retains_live_zero_capacity_read_contract() {
        let mut sensor = Magnetometer::new();
        for input in [&[][..], &[0, 0, 1][..], &[2, 0, 1, 0][..]] {
            assert!(
                !sensor
                    .device_io_control(IOCTL_MFS_READ_REGISTERS, input, 8, true)
                    .success
            );
        }
        assert!(
            !sensor
                .device_io_control(IOCTL_MFS_WRITE_REGISTERS, &[0, 1, 0, 0], 0, false)
                .success
        );
        assert!(
            !sensor
                .device_io_control(IOCTL_MFS_READ_REGISTERS, &[0, 0, 0, 0], 8, true)
                .success
        );
        assert!(
            !sensor
                .device_io_control(IOCTL_MFS_READ_REGISTERS, &[0, 0, 4, 0], 3, true)
                .success
        );
        assert!(
            !sensor
                .device_io_control(IOCTL_MFS_READ_REGISTERS, &[0, 0, 4, 0], 0, false)
                .success
        );
        assert!(
            sensor
                .device_io_control(IOCTL_MFS_READ_REGISTERS, &[0, 0xff, 4, 0], 0, true)
                .success
        );
        assert_eq!(read(&mut sensor, 0, 255).len(), 255);
    }
}
