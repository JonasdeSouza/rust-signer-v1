// use bitcoin::consensus::{deserialize, encode, serialize};
// use bitcoin::psbt::{self, serialize, Error};
// use bitcoin::{Psbt, Transaction};
// extern crate bitcoin;
// use hex;

// fn read_psbt_from_string(psbt_data: &str) -> Result<Psbt, Error> {
//     let psbt_bytes = base64::decode(psbt_data).expect("Failed to decode base64 PSBT");
//     let psbt = Psbt::deserialize(&psbt_bytes).unwrap();

//     Ok(psbt)
// }

// fn decode_psbt(psbt: &bitcoin::Psbt) -> String {
//     format!("{:?}", psbt)
// }

// fn add_info_to_transaction(tx: &mut Transaction, info: &str) {
//     // This is a placeholder function. You can add your logic to modify the transaction here.
//     // For example, you might want to add an output or modify an input.
//     println!("Adding info: {} to transaction: {:?}", info, tx);
// }

// fn transaction_to_psbt(tx: &Transaction) -> Psbt {
//     Psbt::from_unsigned_tx(tx.clone()).expect("Failed to create PSBT")
// }

// pub fn sig_example() {
//     // Example usage
//     let psbt_data = "70736274ff0100750200000001268171371edff285e937adeea4b37b78000c0566cbb3ad64641713ca42171bf60000000000feffffff02d3dff505000000001976a914d0c59903c5bac2868760e90fd521a4665aa7652088ac00e1f5050000000017a9143545e6e33b832c47050f24d3eeb93c9c03948bc787b32e1300000100fda5010100000000010289a3c71eab4d20e0371bbba4cc698fa295c9463afa2e397f8533ccb62f9567e50100000017160014be18d152a9b012039daf3da7de4f53349eecb985ffffffff86f8aa43a71dff1448893a530a7237ef6b4608bbb2dd2d0171e63aec6a4890b40100000017160014fe3e9ef1a745e974d902c4355943abcb34bd5353ffffffff0200c2eb0b000000001976a91485cff1097fd9e008bb34af709c62197b38978a4888ac72fef84e2c00000017a914339725ba21efd62ac753a9bcd067d6c7a6a39d05870247304402202712be22e0270f394f568311dc7ca9a68970b8025fdd3b240229f07f8a5f3a240220018b38d7dcd314e734c9276bd6fb40f673325bc4baa144c800d2f2f02db2765c012103d2e15674941bad4a996372cb87e1856d3652606d98562fe39c5e9e7e413f210502483045022100d12b852d85dcd961d2f5f4ab660654df6eedcc794c0c33ce5cc309ffb5fce58d022067338a8e0e1725c197fb1a88af59f51e44e4255b20167c8684031c05d1f2592a01210223b72beef0965d10be0778efecd61fcac6f79a4ea169393380734464f84f2ab300000000000000";
//     let psbt = read_psbt_from_string(psbt_data).expect("Failed to read PSBT");

//     println!("Decoded PSBT: {}", decode_psbt(&psbt));

//     let mut tx = psbt.unsigned_tx.clone();
//     add_info_to_transaction(&mut tx, "example info");

//     let new_psbt = transaction_to_psbt(&tx);
//     let new_psbt_base64 = base64::encode(&new_psbt.serialize());
//     println!("New PSBT: {}", new_psbt_base64);
// }

use base64;
use base64::Engine;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use bitcoin::consensus::encode;
//use bitcoin::psbt::PartiallySignedTransaction as Psbt;
use bitcoin::Psbt;
use bitcoin::Transaction;
use hex;

fn read_psbt_from_string(psbt_data: &[u8]) -> Result<Psbt, Box<dyn std::error::Error>> {
    //let b64 = general_purpose::STANDARD.decode(psbt_data).unwrap();
    // let psbt: Psbt = Psbt::deserialize(&psbt_bytes)?;
    let psbt = Psbt::deserialize(psbt_data)?;

    Ok(psbt)
}

fn decode_psbt(psbt: &Psbt) -> String {
    format!("{:?}", psbt)
}

fn add_info_to_transaction(tx: &mut Transaction, info: &str) {
    // This is a placeholder function. You can add your logic to modify the transaction here.
    // For example, you might want to add an output or modify an input.
    println!("Adding info: {} to transaction: {:?}", info, tx);
}

fn transaction_to_psbt(tx: &Transaction) -> Result<Psbt, Box<dyn std::error::Error>> {
    let psbt = Psbt::from_unsigned_tx(tx.clone())?;
    Ok(psbt)
}

pub fn sig_example() {
    // Example usage
    let psbt_data = "70736274ff0100750200000001268171371edff285e937adeea4b37b78000c0566cbb3ad64641713ca42171bf60000000000feffffff02d3dff505000000001976a914d0c59903c5bac2868760e90fd521a4665aa7652088ac00e1f5050000000017a9143545e6e33b832c47050f24d3eeb93c9c03948bc787b32e1300000100fda5010100000000010289a3c71eab4d20e0371bbba4cc698fa295c9463afa2e397f8533ccb62f9567e50100000017160014be18d152a9b012039daf3da7de4f53349eecb985ffffffff86f8aa43a71dff1448893a530a7237ef6b4608bbb2dd2d0171e63aec6a4890b40100000017160014fe3e9ef1a745e974d902c4355943abcb34bd5353ffffffff0200c2eb0b000000001976a91485cff1097fd9e008bb34af709c62197b38978a4888ac72fef84e2c00000017a914339725ba21efd62ac753a9bcd067d6c7a6a39d05870247304402202712be22e0270f394f568311dc7ca9a68970b8025fdd3b240229f07f8a5f3a240220018b38d7dcd314e734c9276bd6fb40f673325bc4baa144c800d2f2f02db2765c012103d2e15674941bad4a996372cb87e1856d3652606d98562fe39c5e9e7e413f210502483045022100d12b852d85dcd961d2f5f4ab660654df6eedcc794c0c33ce5cc309ffb5fce58d022067338a8e0e1725c197fb1a88af59f51e44e4255b20167c8684031c05d1f2592a01210223b72beef0965d10be0778efecd61fcac6f79a4ea169393380734464f84f2ab300000000000000";
    
    // Decode hex string to bytes
    let bytes = match hex::decode(psbt_data) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to decode hex: {}", e);
            return;
        }
    };
    
    // Now use the decoded bytes
    let psbt = match read_psbt_from_string(&bytes) {
        Ok(psbt) => psbt,
        Err(e) => {
            eprintln!("Failed to read PSBT: {}", e);
            return;
        }
    };
    
    println!("Decoded PSBT: {}", decode_psbt(&psbt));
    let mut tx = psbt.unsigned_tx.clone();
    add_info_to_transaction(&mut tx, "example info");
    let new_psbt = match transaction_to_psbt(&tx) {
        Ok(psbt) => psbt,
        Err(e) => {
            eprintln!("Failed to create PSBT: {}", e);
            return;
        }
    };
    let new_psbt_base64 = base64::engine::general_purpose::STANDARD.encode(&new_psbt.serialize());
    println!("New PSBT: {}", new_psbt_base64);
}
