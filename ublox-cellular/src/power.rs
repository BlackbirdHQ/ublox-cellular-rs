use core::convert::TryInto;

use atat::AtatClient;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_time::{duration::*, Clock};

use crate::{
    client::Device,
    command::{
        mobile_control::{
            types::{Functionality, ResetMode},
            ModuleSwitchOff, SetModuleFunctionality,
        },
        system_features::{
            types::{FSFactoryRestoreType, NVMFactoryRestoreType},
            SetFactoryConfiguration,
        },
        AT,
    },
    error::{Error, GenericError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum PowerState {
    Off,
    On,
}

impl<C, CLK, RST, DTR, PWR, VINT, const N: usize, const L: usize>
    Device<C, CLK, RST, DTR, PWR, VINT, N, L>
where
    C: AtatClient,
    CLK: Clock,
    RST: OutputPin,
    PWR: OutputPin,
    DTR: OutputPin,
    VINT: InputPin,
{
    /// Check that the cellular module is alive.
    ///
    /// See if the cellular module is responding at the AT interface by poking
    /// it with "AT" up to `attempts` times, waiting 1 second for an "OK"
    /// response each time
    pub(crate) fn is_alive(&mut self, attempts: u8) -> Result<(), Error> {
        let mut error = Error::BaudDetection;
        for _ in 0..attempts {
            match self.network.at_tx.send_ignore_timeout(&AT) {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => error = e.into(),
            };
        }
        Err(error)
    }

    /// Perform at full factory reset of the module, clearing all NVM sectors in the process
    pub fn factory_reset(&mut self) -> Result<(), Error>
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        self.network.send_internal(
            &SetFactoryConfiguration {
                fs_op: FSFactoryRestoreType::AllFiles,
                nvm_op: NVMFactoryRestoreType::NVMFlashSectors,
            },
            false,
        )?;

        defmt::info!("Successfully factory reset modem!");

        if self.soft_reset(true).is_err() {
            self.hard_reset()?;
        }

        Ok(())
    }

    /// Reset the module by sending AT CFUN command
    pub(crate) fn soft_reset(&mut self, sim_reset: bool) -> Result<(), Error>
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        defmt::trace!(
            "Attempting to soft reset of the modem with sim reset: {}.",
            sim_reset
        );

        let fun = if sim_reset {
            Functionality::SilentResetWithSimReset
        } else {
            Functionality::SilentReset
        };

        self.network.send_internal(
            &SetModuleFunctionality {
                fun,
                rst: Some(ResetMode::DontReset),
            },
            false,
        )?;

        self.wait_power_state(PowerState::On, 30_000u32.milliseconds())?;

        Ok(())
    }

    /// Reset the module by driving it's RESET_N pin low for 50 ms
    ///
    /// **NOTE** This function will reset NVM settings!
    pub fn hard_reset(&mut self) -> Result<(), Error>
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        defmt::trace!("Attempting to hard reset of the modem.");
        match self.config.rst_pin {
            Some(ref mut rst) => {
                // Apply Low pulse on RESET_N for 50 milliseconds to reset
                rst.try_set_low().ok();

                self.network
                    .status
                    .timer
                    .new_timer(50.milliseconds())
                    .start()?
                    .wait()?;

                rst.try_set_high().ok();
                self.network
                    .status
                    .timer
                    .new_timer(1.seconds())
                    .start()?
                    .wait()?;
            }
            None => {}
        }

        self.power_state = PowerState::Off;

        self.power_on()?;

        Ok(())
    }

    pub fn power_on(&mut self) -> Result<(), Error>
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        defmt::info!(
            "Attempting to power on the modem with PWR_ON pin: {=bool} and VInt pin: {=bool}.",
            self.config.pwr_pin.is_some(),
            self.config.vint_pin.is_some(),
        );

        if self.power_state()? != PowerState::On {
            defmt::trace!("Powering modem on.");
            match self.config.pwr_pin {
                // Apply Low pulse on PWR_ON for 50 microseconds to power on
                Some(ref mut pwr) => {
                    pwr.try_set_low().ok();
                    self.network
                        .status
                        .timer
                        .new_timer(50.microseconds())
                        .start()?
                        .wait()?;
                    pwr.try_set_high().ok();
                    self.network
                        .status
                        .timer
                        .new_timer(1.seconds())
                        .start()?
                        .wait()?;

                    if let Err(e) = self.wait_power_state(PowerState::On, 5_000u32.milliseconds()) {
                        defmt::error!("Failed to power on modem");
                        return Err(e);
                    } else {
                        defmt::trace!("Modem powered on");
                    }
                }
                _ => {
                    // Software restart
                    if self.soft_reset(false).is_err() {
                        self.hard_reset()?;
                    }
                }
            }
        } else {
            defmt::debug!("module is already on");
        }
        Ok(())
    }

    pub fn soft_power_off(&mut self) -> Result<(), Error> {
        defmt::trace!("Attempting to soft power off the modem.");

        self.network.send_internal(&ModuleSwitchOff, false)?;

        self.power_state = PowerState::Off;
        defmt::trace!("Modem powered off");

        self.network
            .status
            .timer
            .new_timer(10.seconds())
            .start()?
            .wait()?;

        Ok(())
    }

    pub fn hard_power_off(&mut self) -> Result<(), Error> {
        defmt::trace!("Attempting to hard power off the modem.");

        if self.power_state()? == PowerState::On {
            match self.config.pwr_pin {
                Some(ref mut pwr) => {
                    // Apply Low pulse on PWR_ON >= 1 second to power off
                    pwr.try_set_low().ok();
                    self.network
                        .status
                        .timer
                        .new_timer(1.seconds())
                        .start()?
                        .wait()?;
                    pwr.try_set_high().ok();

                    self.power_state = PowerState::Off;
                    defmt::trace!("Modem powered off");

                    self.network
                        .status
                        .timer
                        .new_timer(10.seconds())
                        .start()?
                        .wait()?;
                }
                _ => {
                    return Err(Error::Generic(GenericError::Unsupported));
                }
            }
        }

        Ok(())
    }

    /// Check the power state of the module, by probing `Vint` pin if available,
    /// fallbacking to checking for AT responses through `is_alive`
    pub fn power_state(&mut self) -> Result<PowerState, Error> {
        match self.config.vint_pin {
            Some(ref mut vint) => {
                if vint
                    .try_is_high()
                    .map_err(|_| Error::Generic(GenericError::Unsupported))?
                {
                    Ok(PowerState::On)
                } else {
                    Ok(PowerState::Off)
                }
            }
            _ => Ok(self.is_alive(2).map_or(PowerState::Off, |_| PowerState::On)),
        }
    }

    /// Wait for the power state to change into `expected`, with a timeout
    fn wait_power_state(
        &mut self,
        expected: PowerState,
        timeout: Milliseconds<u32>,
    ) -> Result<(), Error>
    where
        Generic<CLK::T>: TryInto<Milliseconds>,
    {
        let now = self.network.status.timer.try_now().unwrap();

        let mut res = false;

        defmt::trace!("Waiting for the modem to reach {}.", expected);
        while self
            .network
            .status
            .timer
            .try_now()
            .ok()
            .and_then(|ms| ms.checked_duration_since(&now))
            .and_then(|dur| dur.try_into().ok())
            .unwrap()
            < timeout
        {
            if self.power_state()? == expected {
                res = true;
                break;
            }

            self.network
                .status
                .timer
                .new_timer(5.milliseconds())
                .start()?
                .wait()?;
        }

        if res {
            defmt::trace!("Success.");
            Ok(())
        } else {
            defmt::error!("Modem never reach {}.", expected);
            Err(Error::Generic(GenericError::Timeout))
        }
    }
}
