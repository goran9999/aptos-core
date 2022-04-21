// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{
        types::{CliConfig, CliError, CliTypedResult},
        utils::prompt_yes,
    },
    op::key::GenerateKey,
};
use aptos_crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use clap::Parser;

/// Tool to initialize current directory for the aptos tool
#[derive(Debug, Parser)]
pub struct InitTool {}

impl InitTool {
    pub async fn execute(self) -> CliTypedResult<()> {
        let mut config = if CliConfig::config_exists()? {
            if !prompt_yes(
                "Aptos already initialized, do you want to overwrite the existing config?",
            ) {
                eprintln!("Exiting...");
                return Ok(());
            }
            CliConfig::load()?
        } else {
            CliConfig::default()
        };

        eprintln!("Enter your private key as a hex literal (0x...) [No input: Generate new key]");
        let input = read_line("Private key")?;
        let input = input.trim();
        let private_key = if input.is_empty() {
            eprintln!("No key given, generating key...");
            GenerateKey::generate_ed25519_in_memory()
        } else {
            Ed25519PrivateKey::from_encoded_string(input)
                .map_err(|err| CliError::UnableToParse("Ed25519PrivateKey", err.to_string()))?
        };
        config.private_key = Some(private_key);
        config.save()?;
        eprintln!("Aptos is now set up!  Run `aptos help` for more information about commands");

        Ok(())
    }
}

/// Reads a line from input
fn read_line(input_name: &'static str) -> CliTypedResult<String> {
    let mut input_buf = String::new();
    let _ = std::io::stdin()
        .read_line(&mut input_buf)
        .map_err(|err| CliError::IO(input_name.to_string(), err))?;

    Ok(input_buf)
}
