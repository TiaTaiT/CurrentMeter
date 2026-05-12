#![cfg_attr(not(test), no_std)]

/// Shared, hardware-agnostic processing state for current measurements.
///
/// This type is intentionally small and `no_std` compatible so the same logic
/// can be reused by the MCU firmware and tested on a host PC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentSample {
    pub milliamps: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterConfig {
    pub offset_milliamps: i32,
    pub scale_microamps_per_lsb: i32,
}

impl Default for MeterConfig {
    fn default() -> Self {
        Self {
            offset_milliamps: 0,
            scale_microamps_per_lsb: 1_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentMeter {
    cfg: MeterConfig,
}

impl CurrentMeter {
    pub const fn new(cfg: MeterConfig) -> Self {
        Self { cfg }
    }

    pub fn from_adc(&self, raw: u16) -> CurrentSample {
        let scaled_microamps = (raw as i32) * self.cfg.scale_microamps_per_lsb;
        CurrentSample {
            milliamps: (scaled_microamps / 1_000) - self.cfg.offset_milliamps,
        }
    }

    pub fn average<'a>(&self, samples: impl IntoIterator<Item = &'a CurrentSample>) -> Option<CurrentSample> {
        let mut count = 0i32;
        let mut total = 0i32;

        for sample in samples {
            count += 1;
            total += sample.milliamps;
        }

        if count == 0 {
            None
        } else {
            Some(CurrentSample {
                milliamps: total / count,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrentMeter, CurrentSample, MeterConfig};

    #[test]
    fn converts_adc_reading_to_milliamps() {
        let meter = CurrentMeter::new(MeterConfig {
            offset_milliamps: 12,
            scale_microamps_per_lsb: 250,
        });

        assert_eq!(meter.from_adc(200), CurrentSample { milliamps: 38 });
    }

    #[test]
    fn computes_average_for_host_side_tests() {
        let meter = CurrentMeter::new(MeterConfig::default());
        let samples = [
            CurrentSample { milliamps: 10 },
            CurrentSample { milliamps: 14 },
            CurrentSample { milliamps: 18 },
        ];

        assert_eq!(meter.average(samples.iter()), Some(CurrentSample { milliamps: 14 }));
    }

    #[test]
    fn returns_none_for_empty_average() {
        let meter = CurrentMeter::new(MeterConfig::default());

        assert_eq!(meter.average([].iter()), None);
    }
}
