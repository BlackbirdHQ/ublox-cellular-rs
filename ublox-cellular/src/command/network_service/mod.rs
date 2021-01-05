//! ### 7 - Network service

mod impl_;
pub mod responses;
pub mod types;
pub mod urc;

use super::NoResponse;
use atat::atat_derive::AtatCmd;
use responses::*;
use types::*;

/// 7.3 Signal quality +CSQ
///
/// Returns the radio signal strength <signal_power> and <qual> from the MT.
///
/// **NOTES:**
/// - **TOBY-L4 / LARA-R2 / TOBY-R2** - The +CSQ utilization is deprecated. It
///   is warmly recommended to use the command +CESQ to obtain the same
///   information more accurately.
/// - **TOBY-L4 / TOBY-L2 / MPCI-L2 / LARA-R2 / TOBY-R2 / SARA-U2 / LISA-U2 /
///   LISA-U1 / SARA-G4 / SARA-G3 / LEON-G1** - The radio signal strength
///   <signal_power> will be also used to build and display the indicator
///   "signal" i.e. signal quality in the information text response of +CIND and
///   in the +CIEV URC (see the +CMER command description).
///
/// In dedicated mode, during the radio channel reconfiguration (e.g. handover),
/// invalid measurements may be returned for a short transitory because the MT
/// must compute them on the newly assigned channel.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CSQ", SignalQuality)]
pub struct GetSignalQuality;

/// 7.5 Operator selection +COPS
#[derive(Clone, AtatCmd)]
#[at_cmd("+COPS", NoResponse, timeout_ms = 180000)]
pub struct SetOperatorSelection {
    #[at_arg(position = 0)]
    pub mode: OperatorSelectionMode,
    // pub format: NetworkRegistrationUrcConfig,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+COPS?", OperatorSelection, timeout_ms = 180000)]
pub struct GetOperatorSelection;

/// 7.8 Radio Access Technology (RAT) selection +URAT Forces the selection of
/// the Radio Access Technology (RAT) in the protocol stack. On the subsequent
/// network registration (+COPS, +CGATT) the selected RAT is used.
///
/// By means of <PreferredAct> and <2ndPreferredAct> parameters it is possible
/// to define the order of RAT selection at boot or when entering full
/// functionality from de-registered state. If <SelectedAcT> is set to dual or
/// tri mode, it is possible to specify the preferred RAT parameter
/// <PreferredAct>, which determines which RAT is selected first. If the
/// preferred RAT parameter is omitted, it will be set by default to the highest
/// RAT in the current multi-mode range. If tri mode is selected, it is also
/// possible to specify a second preferred RAT <2ndPreferredAct> in addition to
/// the preferred RAT. This parameter determines which RAT is selected if no
/// cellular service can be obtained by the module on the preferred RAT. The
/// remaining RAT will be selected when no service can be obtained in the
/// preferred one(s).
///
/// **NOTES:**
/// - Any change in the RAT selection must be done deregistered state, entered
///   by issuing the AT+CFUN=0 or AT+CFUN=4 AT command. Use AT+CFUN=1 to return
///   to the module full functionality.
/// - **SARA-U2 / LISA-U2 / LISA-U1** - See Notes for the procedure to enter the
///   detach state.
/// - u-blox cellular modules are certified according to all the capabilities
///   and options stated in the Protocol Implementation Conformance Statement
///   document (PICS) of the module. The PICS, according to 3GPP TS 51.010-2
///   [66], 3GPP TS 34.121-2 [67], 3GPP TS 36.521-2 [94] and 3GPP TS 36.523-2
///   [95], is a statement of the implemented and supported capabilities and
///   options of a device. If the user changes the command settings during the
///   certification process, the PICS of the application device integrating a
///   u-blox cellular module must be changed accordingly.
/// - **TOBY-L4 / TOBY-L2 / MPCI-L2 / LARA-R2 / TOBY-R2 / SARA-U2 / LISA-U2 /
///   LISA-U1** - In dual mode and tri mode, all the requested Access Stratum
///   protocols are active and Inter-RAT measurements as well as Inter-RAT
///   handovers may be performed (if ordered by the network).
/// - **TOBY-L200 / TOBY-L201 / MPCI-L200 / MPCI-L201 / LARA-R202-02B /
///   LARA-R203 / TOBY-R200-02B / TOBY-R202** - AT&T RAT balancing feature, by
///   means of updating RAT related SIM files, can force RAT usage (see Notes).
#[derive(Clone, AtatCmd)]
#[at_cmd("+URAT", NoResponse)]
pub struct SetRadioAccessTechnology {
    #[at_arg(position = 0)]
    pub selected_act: RadioAccessTechnologySelected,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+URAT?", RadioAccessTechnology)]
pub struct GetRadioAccessTechnology;

/// 7.14 Network registration status +CREG
///
/// Configures the network registration URC related to CS domain. Depending on the <n> parameter value, a URC
/// can be issued:
/// • +CREG: <stat> if <n>=1 and there is a change in the MT's circuit switched mode network registration status
/// in GERAN/UTRAN/E-UTRAN
/// • +CREG: <stat>[,<lac>,<ci>[,<AcTStatus>]] if <n>=2 and there is a change of the network cell in GERAN/
/// UTRAN/E-UTRAN
/// The parameters <AcTStatus>, <lac>, <ci> are provided only if available.
/// The read command provides the same information issued by the URC together with the current value of the
/// <n> parameter. The location information elements <lac>, <ci> and <AcTStatus>, if available, are returned only
/// when <n>=2 and the MT is registered with the network.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CREG", NoResponse)]
pub struct SetNetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: NetworkRegistrationUrcConfig,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CREG?", NetworkRegistrationStatus)]
pub struct GetNetworkRegistrationStatus;
