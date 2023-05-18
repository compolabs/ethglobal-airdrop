use std::io::Write;

use fuels::{
    prelude::{abigen, Contract, LoadConfiguration, TxParameters, WalletUnlocked},
    types::{Address, AssetId, ContractId, SizedAsciiString},
};

pub struct DeployTokenConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub mint_amount: u64,
}

pub struct Asset {
    pub config: DeployTokenConfig,
    pub contract_id: ContractId,
    pub asset_id: AssetId,
    pub instance: Option<TokenContract<WalletUnlocked>>,
    pub default_price: u64,
}

abigen!(Contract(
    name = "TokenContract",
    abi = "tests/artefacts/token/token_contract-abi.json"
));

pub mod token_abi_calls {

    use fuels::{prelude::TxParameters, programs::call_response::FuelCallResponse, types::Address};

    use super::*;

    pub async fn mint(c: &TokenContract<WalletUnlocked>) -> FuelCallResponse<()> {
        let res = c.methods().mint().append_variable_outputs(1).call().await;
        res.unwrap()
    }
    pub async fn mint_and_transfer(
        c: &TokenContract<WalletUnlocked>,
        amount: u64,
        recipient: Address,
    ) -> FuelCallResponse<()> {
        c.methods()
            .mint_and_transfer(amount, recipient)
            .append_variable_outputs(1)
            .tx_params(TxParameters::default().set_gas_price(1))
            .call()
            .await
            .unwrap()
    }
    pub async fn initialize(
        c: &TokenContract<WalletUnlocked>,
        config: TokenInitializeConfig,
        mint_amount: u64,
        address: Address,
    ) -> FuelCallResponse<()> {
        c.methods()
            .initialize(config, mint_amount, address)
            .call()
            .await
            .expect("❌ Cannot initialize token")
    }
}

pub async fn get_token_contract_instance(
    wallet: &WalletUnlocked,
    deploy_config: &DeployTokenConfig,
) -> TokenContract<WalletUnlocked> {
    let mut name = deploy_config.name.clone();
    let mut symbol = deploy_config.symbol.clone();
    let decimals = deploy_config.decimals;

    let mut salt: [u8; 32] = [0; 32];
    let mut temp: &mut [u8] = &mut salt;
    temp.write(symbol.clone().as_bytes()).unwrap();

    let id = Contract::load_from(
        "./tests/artefacts/token/token_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .with_salt(salt)
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();
    let instance = TokenContract::new(id, wallet.clone());

    name.push_str(" ".repeat(32 - deploy_config.name.len()).as_str());
    symbol.push_str(" ".repeat(8 - deploy_config.symbol.len()).as_str());

    let config: TokenInitializeConfig = TokenInitializeConfig {
        name: SizedAsciiString::<32>::new(name).unwrap(),
        symbol: SizedAsciiString::<8>::new(symbol).unwrap(),
        decimals,
    };

    let address = Address::from(wallet.address());
    token_abi_calls::initialize(&instance, config, 1, address).await;
    token_abi_calls::mint(&instance).await;

    instance
}
