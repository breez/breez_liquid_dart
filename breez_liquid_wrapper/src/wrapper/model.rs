use boltz_client::network::Chain;
use boltz_client::Bolt11Invoice;
use lwk_signer::SwSigner;
use lwk_wollet::{ElectrumUrl, ElementsNetwork, WolletDescriptor};
use serde::Serialize;

use crate::get_invoice_amount;

use super::error::LsSdkError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Network {
    Liquid,
    LiquidTestnet,
}

impl From<Network> for ElementsNetwork {
    fn from(value: Network) -> Self {
        match value {
            Network::Liquid => ElementsNetwork::Liquid,
            Network::LiquidTestnet => ElementsNetwork::LiquidTestnet,
        }
    }
}

impl From<Network> for Chain {
    fn from(value: Network) -> Self {
        match value {
            Network::Liquid => Chain::Liquid,
            Network::LiquidTestnet => Chain::LiquidTestnet,
        }
    }
}

#[derive(Debug)]
pub struct WalletOptions {
    pub signer: SwSigner,
    pub network: Network,
    /// Output script descriptor
    ///
    /// See <https://github.com/bitcoin/bips/pull/1143>
    pub descriptor: WolletDescriptor,
    /// Absolute or relative path to the data dir, including the dir name.
    ///
    /// If not set, it defaults to [crate::DEFAULT_DATA_DIR].
    pub data_dir_path: Option<String>,
    /// Custom Electrum URL. If set, it must match the specified network.
    ///
    /// If not set, it defaults to a Blockstream instance.
    pub electrum_url: Option<ElectrumUrl>,
}
impl WalletOptions {
    pub(crate) fn get_electrum_url(&self) -> ElectrumUrl {
        self.electrum_url.clone().unwrap_or({
            let (url, validate_domain, tls) = match &self.network {
                Network::Liquid => ("blockstream.info:995", true, true),
                Network::LiquidTestnet => ("blockstream.info:465", true, true),
            };
            ElectrumUrl::new(url, tls, validate_domain)
        })
    }
}

#[derive(Debug, Serialize)]
pub struct PrepareReceiveRequest {
    pub payer_amount_sat: Option<u64>,
    pub receiver_amount_sat: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PrepareReceiveResponse {
    pub pair_hash: String,
    pub payer_amount_sat: u64,
    pub fees_sat: u64,
}

#[derive(Debug, Serialize)]
pub struct ReceivePaymentResponse {
    pub id: String,
    pub invoice: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PrepareSendResponse {
    pub id: String,
    pub payer_amount_sat: u64,
    pub receiver_amount_sat: u64,
    pub total_fees: u64,
    pub funding_address: String,
    pub invoice: String,
}

#[derive(Debug, Serialize)]
pub struct SendPaymentResponse {
    pub txid: String,
}

#[derive(thiserror::Error, Debug)]
pub enum PaymentError {
    #[error("Invoice amount is out of range")]
    AmountOutOfRange,

    #[error("The specified funds have already been claimed")]
    AlreadyClaimed,

    #[error("Generic error: {err}")]
    Generic { err: String },

    #[error("The specified invoice is not valid")]
    InvalidInvoice,

    #[error("The generated preimage is not valid")]
    InvalidPreimage,

    #[error("Lwk error: {err}")]
    LwkError { err: String },

    #[error("Boltz did not return any pairs from the request")]
    PairsNotFound,

    #[error("Could not store the swap details locally")]
    PersistError,

    #[error("Could not sign/send the transaction: {err}")]
    SendError { err: String },

    #[error("Could not sign the transaction: {err}")]
    SignerError { err: String },
}

impl From<boltz_client::error::Error> for PaymentError {
    fn from(err: boltz_client::error::Error) -> Self {
        match err {
            boltz_client::error::Error::Protocol(msg) => {
                if msg == "Could not find utxos for script" {
                    return PaymentError::AlreadyClaimed;
                }

                PaymentError::Generic { err: msg }
            }
            _ => PaymentError::Generic {
                err: format!("{err:?}"),
            },
        }
    }
}

#[allow(clippy::match_single_binding)]
impl From<lwk_wollet::Error> for PaymentError {
    fn from(err: lwk_wollet::Error) -> Self {
        match err {
            _ => PaymentError::LwkError {
                err: format!("{err:?}"),
            },
        }
    }
}

#[allow(clippy::match_single_binding)]
impl From<lwk_signer::SignerError> for PaymentError {
    fn from(err: lwk_signer::SignerError) -> Self {
        match err {
            _ => PaymentError::SignerError {
                err: format!("{err:?}"),
            },
        }
    }
}

impl From<anyhow::Error> for PaymentError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic {
            err: err.to_string(),
        }
    }
}

impl From<LsSdkError> for PaymentError {
    fn from(err: LsSdkError) -> Self {
        Self::Generic {
            err: err.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub balance_sat: u64,
    pub pubkey: String,
}

#[derive(Debug)]
pub(crate) enum OngoingSwap {
    Send {
        id: String,
        funding_address: String,
        invoice: String,
        receiver_amount_sat: u64,
        txid: Option<String>,
    },
    Receive {
        id: String,
        preimage: String,
        redeem_script: String,
        blinding_key: String,
        invoice: String,
        receiver_amount_sat: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PaymentType {
    Sent,
    Received,
    PendingReceive,
    PendingSend,
}

#[derive(Debug, Clone, Serialize)]
pub struct Payment {
    pub id: Option<String>,
    pub timestamp: Option<u32>,
    pub amount_sat: u64,
    pub fees_sat: Option<u64>,
    #[serde(rename(serialize = "type"))]
    pub payment_type: PaymentType,

    /// Only for [PaymentType::PendingReceive]
    pub invoice: Option<String>,
}

impl From<OngoingSwap> for Payment {
    fn from(swap: OngoingSwap) -> Self {
        match swap {
            OngoingSwap::Send {
                invoice,
                receiver_amount_sat,
                ..
            } => {
                let payer_amount_sat = get_invoice_amount!(invoice);
                Payment {
                    id: None,
                    timestamp: None,
                    payment_type: PaymentType::PendingSend,
                    amount_sat: payer_amount_sat,
                    invoice: Some(invoice),
                    fees_sat: Some(receiver_amount_sat - payer_amount_sat),
                }
            }
            OngoingSwap::Receive {
                receiver_amount_sat,
                invoice,
                ..
            } => {
                let payer_amount_sat = get_invoice_amount!(invoice);
                Payment {
                    id: None,
                    timestamp: None,
                    payment_type: PaymentType::PendingReceive,
                    amount_sat: receiver_amount_sat,
                    invoice: Some(invoice),
                    fees_sat: Some(payer_amount_sat - receiver_amount_sat),
                }
            }
        }
    }
}

pub(crate) struct PaymentData {
    pub payer_amount_sat: u64,
}

#[macro_export]
macro_rules! get_invoice_amount {
    ($invoice:expr) => {
        $invoice
            .parse::<Bolt11Invoice>()
            .expect("Expecting valid invoice")
            .amount_milli_satoshis()
            .expect("Expecting valid amount")
            / 1000
    };
}
