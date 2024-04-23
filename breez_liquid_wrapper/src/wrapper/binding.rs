use std::sync::Arc;

use std::future::Future;

use anyhow::Result;
use boltz_client::util::secrets::LBtcReverseRecovery;

use crate::wrapper::{
    error::LsSdkError,
    model::{
        Network, PaymentError, PrepareReceiveRequest, PrepareReceiveResponse, PrepareSendResponse,
        ReceivePaymentResponse, SendPaymentResponse, WalletInfo,
    },
    wallet::Wallet,
};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use super::model::Payment;

pub fn init(mnemonic: String, data_dir: Option<String>, network: Network) -> Result<()> {
    block_on(async move {
        let mut locked = WALLET_INSTANCE.lock().await;
        match *locked {
            None => {
                let wallet = Wallet::init(&mnemonic, data_dir, network)?;

                *locked = Some(wallet);
                core::result::Result::Ok(())
            }
            Some(_) => Err(LsSdkError::Generic {
                err: "Static node services already set, please call disconnect() first".into(),
            }),
        }
    })
    .map_err(anyhow::Error::new::<LsSdkError>)
}

pub fn get_info(with_scan: bool) -> Result<WalletInfo, LsSdkError> {
    block_on(async { get_wallet().await?.get_info(with_scan).map_err(Into::into) })
}

pub fn prepare_send_payment(invoice: String) -> Result<PrepareSendResponse, PaymentError> {
    block_on(async { get_wallet().await?.prepare_send_payment(&invoice) })
}

pub fn send_payment(req: PrepareSendResponse) -> Result<SendPaymentResponse, PaymentError> {
    block_on(async { get_wallet().await?.send_payment(&req) })
}

pub fn prepare_receive_payment(
    req: PrepareReceiveRequest,
) -> Result<PrepareReceiveResponse, PaymentError> {
    block_on(async { get_wallet().await?.prepare_receive_payment(&req) })
}

pub fn receive_payment(
    req: PrepareReceiveResponse,
) -> Result<ReceivePaymentResponse, PaymentError> {
    block_on(async { get_wallet().await?.receive_payment(&req) })
}

pub fn list_payments(with_scan: bool, include_pending: bool) -> Result<Vec<Payment>> {
    block_on(async {
        get_wallet()
            .await?
            .list_payments(with_scan, include_pending)
    })
}

pub fn recover_funds(recovery: LBtcReverseRecovery) -> Result<String> {
    block_on(async { get_wallet().await?.recover_funds(&recovery) })
}

pub fn empty_wallet_cache() -> Result<()> {
    block_on(async { get_wallet().await?.empty_wallet_cache() })
}

/*
The format Lazy<Mutex<Option<...>>> for the following variables allows them to be instance-global,
meaning they can be set only once per instance, but calling disconnect() will unset them.
 */
static WALLET_INSTANCE: Lazy<Mutex<Option<Arc<Wallet>>>> = Lazy::new(|| Mutex::new(None));
static RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

async fn get_wallet() -> Result<Arc<Wallet>, LsSdkError> {
    match WALLET_INSTANCE.lock().await.as_ref() {
        None => Err(LsSdkError::Generic {
            err: "Liquid wallet was not initialized".into(),
        }),
        Some(ln_sdk) => core::result::Result::Ok(ln_sdk.clone()),
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    rt().block_on(future)
}

pub(crate) fn rt() -> &'static tokio::runtime::Runtime {
    &RT
}
