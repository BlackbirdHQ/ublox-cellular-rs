use crate::command::{
    network_service::{
        responses::NetworkRegistrationStatus,
        types::{NetworkRegistrationStat, RatAct},
        // urc::NetworkRegistration,
    },
    psn::{
        responses::{EPSNetworkRegistrationStatus, GPRSNetworkRegistrationStatus},
        types::{EPSNetworkRegistrationStat, GPRSNetworkRegistrationStat},
        // urc::{EPSNetworkRegistration, GPRSNetworkRegistration},
    },
};
use embassy_time::{Duration, Instant};
use heapless::String;

#[derive(Debug, Clone, Default)]
pub struct CellularRegistrationStatus {
    status: Status,
    updated: Option<Instant>,
    started: Option<Instant>,
}

impl CellularRegistrationStatus {
    pub fn new() -> Self {
        Self {
            status: Status::default(),
            updated: None,
            started: None,
        }
    }

    pub fn duration(&self, ts: Instant) -> Duration {
        self.started
            .and_then(|started| ts.checked_duration_since(started))
            .unwrap_or_else(|| Duration::from_millis(0))
    }

    pub fn started(&self) -> Option<Instant> {
        self.started
    }

    pub fn updated(&self) -> Option<Instant> {
        self.updated
    }

    pub fn reset(&mut self) {
        self.status = Status::None;
        self.updated = None;
        self.started = None;
    }

    pub fn get_status(&self) -> Status {
        self.status
    }

    pub fn set_status(&mut self, stat: Status) {
        let ts = Instant::now();
        if self.status != stat {
            self.status = stat;
            self.started = Some(ts);
        }
        self.updated = Some(ts);
    }

    pub fn registered(&self) -> bool {
        matches!(self.status, Status::Home | Status::Roaming)
    }

    pub fn sticky(&self) -> bool {
        self.updated.is_some() && self.updated != self.started
    }
}

impl From<u8> for Status {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::NotRegistering,
            1 => Self::Home,
            2 => Self::Searching,
            3 => Self::Denied,
            4 => Self::Unknown,
            5 => Self::Roaming,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Status {
    None,
    NotRegistering,
    Home,
    Searching,
    Denied,
    Unknown,
    Roaming,
}

impl Default for Status {
    fn default() -> Self {
        Self::None
    }
}

/// Convert the 3GPP registration status from a CREG URC to [`RegistrationStatus`].
impl From<NetworkRegistrationStat> for Status {
    fn from(v: NetworkRegistrationStat) -> Self {
        Self::from(v as u8)
    }
}

/// Convert the 3GPP registration status from a CGREG URC to [`RegistrationStatus`].
impl From<GPRSNetworkRegistrationStat> for Status {
    fn from(v: GPRSNetworkRegistrationStat) -> Self {
        Self::from(v as u8)
    }
}

/// Convert the 3GPP registration status from a CEREG URC to [`RegistrationStatus`].
impl From<EPSNetworkRegistrationStat> for Status {
    fn from(v: EPSNetworkRegistrationStat) -> Self {
        Self::from(v as u8)
    }
}

#[derive(Debug, Default)]
pub struct RegistrationParams {
    reg_type: RegType,
    pub(crate) status: Status,
    act: RatAct,

    cell_id: Option<String<8>>,
    lac: Option<String<4>>,
    // active_time: Option<u16>,
    // periodic_tau: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegType {
    Creg,
    Cgreg,
    Cereg,
    Unknown,
}

impl Default for RegType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<RadioAccessNetwork> for RegType {
    fn from(ran: RadioAccessNetwork) -> Self {
        match ran {
            RadioAccessNetwork::UnknownUnused => Self::Unknown,
            RadioAccessNetwork::Geran => Self::Creg,
            RadioAccessNetwork::Utran => Self::Cgreg,
            RadioAccessNetwork::Eutran => Self::Cereg,
        }
    }
}

impl From<RegType> for RadioAccessNetwork {
    fn from(regtype: RegType) -> Self {
        match regtype {
            RegType::Unknown => Self::UnknownUnused,
            RegType::Creg => Self::Geran,
            RegType::Cgreg => Self::Utran,
            RegType::Cereg => Self::Eutran,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CellularGlobalIdentity {
    /// Registered network operator cell Id.
    cell_id: Option<String<8>>,
    /// Registered network operator Location Area Code.
    lac: Option<String<4>>,
    // Registered network operator Routing Area Code.
    // rac: u8,
    // Registered network operator Tracking Area Code.
    // tac: u8,
}

#[derive(Debug, Clone)]
pub struct RegistrationState {
    pub(crate) reg_check_time: Option<Instant>,
    pub(crate) reg_start_time: Option<Instant>,
    pub(crate) imsi_check_time: Option<Instant>,

    pub(crate) conn_state: ConnectionState,
    /// CSD (Circuit Switched Data) registration status (registered/searching/roaming etc.).
    pub(crate) csd: CellularRegistrationStatus,
    /// PSD (Packet Switched Data) registration status (registered/searching/roaming etc.).
    pub(crate) psd: CellularRegistrationStatus,
    /// EPS (Evolved Packet Switched) registration status (registered/searching/roaming etc.).
    pub(crate) eps: CellularRegistrationStatus,

    pub(crate) registration_interventions: u8,
    check_imsi: bool,

    pub(crate) cgi: CellularGlobalIdentity,
    // Radio Access Technology (RAT)
    // pub(crate) act: RatAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl RegistrationState {
    pub fn new() -> Self {
        Self {
            reg_check_time: None,
            reg_start_time: None,
            imsi_check_time: None,

            conn_state: ConnectionState::Disconnected,
            csd: CellularRegistrationStatus::new(),
            psd: CellularRegistrationStatus::new(),
            eps: CellularRegistrationStatus::new(),
            registration_interventions: 1,
            check_imsi: false,

            cgi: CellularGlobalIdentity::default(),
            // act: RatAct::default(),
        }
    }

    pub fn reset(&mut self) {
        self.csd.reset();
        self.psd.reset();
        self.eps.reset();
        self.reg_start_time = Some(Instant::now());
        self.reg_check_time = Some(Instant::now());
        self.imsi_check_time = None;
        self.registration_interventions = 1;
    }

    pub fn set_connection_state(&mut self, state: ConnectionState) {
        if self.conn_state == state {
            return;
        }

        trace!("Connection state changed to \"{:?}\"", state);
        self.conn_state = state;
    }

    pub fn compare_and_set(&mut self, new_params: RegistrationParams) {
        match new_params.reg_type {
            RegType::Creg => {
                let prev_reg_status = self.csd.registered();
                self.csd.set_status(new_params.status);
                if !prev_reg_status && self.csd.registered() {
                    self.check_imsi = true
                }
            }
            RegType::Cgreg => {
                let prev_reg_status = self.psd.registered();
                self.psd.set_status(new_params.status);
                if !prev_reg_status && self.psd.registered() {
                    self.check_imsi = true
                }
            }
            RegType::Cereg => {
                let prev_reg_status = self.eps.registered();
                self.eps.set_status(new_params.status);
                if !prev_reg_status && self.eps.registered() {
                    self.check_imsi = true
                }
            }
            RegType::Unknown => {
                error!("unknown reg type");
                return;
            }
        }

        // Update Cellular Global Identity
        if new_params.cell_id.is_some() && self.cgi.cell_id != new_params.cell_id {
            self.cgi.cell_id = new_params.cell_id.clone();
            self.cgi.lac = new_params.lac;
        }
    }
}

// impl From<NetworkRegistration> for RegistrationParams {
//     fn from(v: NetworkRegistration) -> Self {
//         Self {
//             act: RatAct::Gsm,
//             reg_type: RegType::Creg,
//             status: v.stat.into(),
//             cell_id: None,
//             lac: None,
//             // active_time: None,
//             // periodic_tau: None,
//         }
//     }
// }

impl From<NetworkRegistrationStatus> for RegistrationParams {
    fn from(v: NetworkRegistrationStatus) -> Self {
        Self {
            act: RatAct::Gsm,
            reg_type: RegType::Creg,
            status: v.stat.into(),
            cell_id: None,
            lac: None,
            // active_time: None,
            // periodic_tau: None,
        }
    }
}

// impl From<GPRSNetworkRegistration> for RegistrationParams {
//     fn from(v: GPRSNetworkRegistration) -> Self {
//         Self {
//             act: v.act.unwrap_or(RatAct::Unknown),
//             reg_type: RegType::Cgreg,
//             status: v.stat.into(),
//             cell_id: v.ci,
//             lac: v.lac,
//             // active_time: None,
//             // periodic_tau: None,
//         }
//     }
// }

impl From<GPRSNetworkRegistrationStatus> for RegistrationParams {
    fn from(v: GPRSNetworkRegistrationStatus) -> Self {
        Self {
            reg_type: RegType::Cgreg,
            status: v.stat.into(),
            cell_id: v.ci,
            lac: v.lac,
            act: v.act.unwrap_or(RatAct::Unknown),
            // active_time: None,
            // periodic_tau: None,
        }
    }
}

// impl From<EPSNetworkRegistration> for RegistrationParams {
//     fn from(v: EPSNetworkRegistration) -> Self {
//         Self {
//             reg_type: RegType::Cereg,
//             status: v.stat.into(),
//             cell_id: v.ci,
//             lac: v.tac,
//             act: v.act.unwrap_or(RatAct::Unknown),
//             // active_time: None,
//             // periodic_tau: None,
//         }
//     }
// }

impl From<EPSNetworkRegistrationStatus> for RegistrationParams {
    fn from(v: EPSNetworkRegistrationStatus) -> Self {
        Self {
            reg_type: RegType::Cereg,
            status: v.stat.into(),
            cell_id: v.ci,
            lac: v.tac,
            act: v.act.unwrap_or(RatAct::Unknown),
            // active_time: None,
            // periodic_tau: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RadioAccessNetwork {
    UnknownUnused = 0,
    Geran = 1,
    Utran = 2,
    Eutran = 3,
}

impl From<usize> for RadioAccessNetwork {
    fn from(v: usize) -> Self {
        match v {
            1 => Self::Geran,
            2 => Self::Utran,
            3 => Self::Eutran,
            _ => Self::UnknownUnused,
        }
    }
}
