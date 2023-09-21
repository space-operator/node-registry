use crate::prelude::*;
use base64::decode;
use mpl_token_auth_rules::payload;
use wormhole_sdk::{nft::Message as NftMessage, token::Message, vaa::Digest, Address, Chain, Vaa};

use super::MessageAlias;

// Command Name
const NAME: &str = "parse_vaa";

const DEFINITION: &str =
    include_str!("../../../../node-definitions/solana/wormhole/parse_vaa.json");

fn build() -> Result<Box<dyn CommandTrait>, CommandError> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Result<CmdBuilder, BuilderError>> =
        Lazy::new(|| CmdBuilder::new(DEFINITION)?.check_name(NAME));
    Ok(CACHE.clone()?.build(run))
}

inventory::submit!(CommandDescription::new(NAME, |_| { build() }));

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub vaa: String,
    // pub vaa_payload_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    parsed_vaa: Vaa<Vec<u8>>,
    vaa_bytes: bytes::Bytes,
    signatures: Vec<wormhole_sdk::vaa::Signature>,
    body: bytes::Bytes,
    vaa_hash: bytes::Bytes,
    vaa_secp256k_hash: bytes::Bytes,
    guardian_set_index: u32,
    payload: serde_json::Value,
}

async fn run(_ctx: Context, input: Input) -> Result<Output, CommandError> {
    let vaa_string = &input.vaa;

    let vaa_bytes = decode(vaa_string)
        .map_err(|err| anyhow::anyhow!("Failed to decode VAA string: {}", err))?;

    let sig_start = 6;
    let num_signers = vaa_bytes[5] as usize;
    let sig_length = 66;

    let mut guardian_signatures = Vec::new();
    for i in 0..num_signers {
        let start = sig_start + i * sig_length;
        let mut signature = [0u8; 65];
        signature.copy_from_slice(&vaa_bytes[start + 1..start + 66]);
        guardian_signatures.push(wormhole_sdk::vaa::Signature {
            index: vaa_bytes[start],
            signature,
        });
    }

    let body = &vaa_bytes[sig_start + sig_length * num_signers..];
    // Check this https://github.com/wormhole-foundation/wormhole/blob/14a1251c06b3d837dcbd2b7bed5b1abae6eb7d02/solana/bridge/program/src/vaa.rs#L176
    let parsed_vaa: Vaa<Vec<u8>> = Vaa {
        version: vaa_bytes[0],
        guardian_set_index: u32::from_be_bytes(
            vaa_bytes[1..5]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert guardian_set_index"))?,
        ),
        signatures: guardian_signatures.clone(),
        timestamp: u32::from_be_bytes(
            body[0..4]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert timestamp"))?,
        ),
        nonce: u32::from_be_bytes(
            body[4..8]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert nonce"))?,
        ),
        emitter_chain: Chain::from(u16::from_be_bytes(
            body[8..10]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert emitter_chain"))?,
        )),
        emitter_address: Address(
            body[10..42]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert emitter_address"))?,
        ),
        sequence: u64::from_be_bytes(
            body[42..50]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert sequence"))?,
        ),
        consistency_level: body[50],
        // gets converted to base64 string?
        payload: body[51..].to_vec(),
    };

    // let (_, body): (Header, Body<Vec<u8>>) = parsed_vaa.into();

    let Digest {
        hash: vaa_hash,
        secp256k_hash: vaa_secp256k_hash,
    } = wormhole_sdk::vaa::digest(body).map_err(|_| anyhow::anyhow!("Failed to digest VAA"))?;

    let payload = match serde_wormhole::from_slice(&parsed_vaa.payload) {
        Ok(message) => MessageAlias::Transfer(message),
        Err(_) => match serde_wormhole::from_slice(&parsed_vaa.payload) {
            Ok(nft_message) => MessageAlias::NftTransfer(nft_message),
            Err(_) => return Err(anyhow::anyhow!("Payload content not supported")),
        },
    };
    let payload: serde_json::Value = serde_json::from_str(&serde_json::to_string(&payload)?)?;

    let payload = payload
        .get("NftTransfer")
        .or(payload.get("Transfer"))
        .ok_or_else(|| anyhow::anyhow!("Invalid payload"))?;

    Ok(Output {
        parsed_vaa: parsed_vaa.clone(),
        vaa_bytes: bytes::Bytes::copy_from_slice(&vaa_bytes),
        signatures: guardian_signatures,
        body: bytes::Bytes::copy_from_slice(body),
        vaa_hash: bytes::Bytes::copy_from_slice(&vaa_hash),
        vaa_secp256k_hash: bytes::Bytes::copy_from_slice(&vaa_secp256k_hash),
        guardian_set_index: parsed_vaa.guardian_set_index,
        payload: payload.clone(),
    })
}

#[test]
fn test() -> Result<(), anyhow::Error> {
    //sol vaa, not supported payload
    let _vaa_string:String = "AQAAAAABAE9eT/T0B917C5+ZQEHdlDUD/b7PNfTkyy/mXX7LPSJzVS6VTJx1gigK7xCic3UywM5/ehtUnZ/HCdoLQtOLX1IBZLYUVg1YsBoAAcARZHHBCI3jyzPKm9l0vBFJ3DJ4Yh+vmP6ZmTrfVHxrAAAAAAAAAAABSGVsbG8gV29ybGQh".to_string();
    let vaa_string:String ="AQAAAAABAMy+FBjMJafK1Xt4cCSbJ03jxJs3f3UW647HrdpT34XWE/7CBbQjo+0xMQXDTlh5IymI6wissEo8TkxTwY/ufCwBZMMBLO/WHgoAATsmQJ+Kre0/XdyhhGlapqD6gpsMhcr4SFYySJbSFMqYAAAAAAAAX3cgAv+98jdq256Gu41IuSzwRBryKQ5Ku3e8LsfhFUYdQ2pkAAEJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string();
    // //eth vaa
    // let vaa_string:String="AQAAAAABAHZle4NbI4+ItAFCCwtKYDthhzq61u1az/gZIbW+hQ8MRskKSDEvutVy7pjuRwRq7EsKhB/lMz4XDDxoeyVm6YkBZMASCPZ6AAAnEgAAAAAAAAAAAAAAANtUkiZfYDiDHon0lWcP+Qmt6UvZAAAAAAAAAZgBAgAAAAAAAAAAAAAAAEEKixUC8B8oh/CwWyLMk01FpiinJxISRVJDX1NZTUJPTAAAAAAAAAAAAAAAAAAAAAAAAAAAAABNeUVSQzIwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string();

    // //eth transfer vaa
    // let vaa_string:String="AQAAAAABAIDirkZb0u0i33P55FM8+ErUor6LbHELePcpfMyC3JRHPFQJ7ztwLOI9XlwvK1cqgSQC8Q+4hh/gyV5W8/rKt2cBZMFSePdTAQAnEgAAAAAAAAAAAAAAANtUkiZfYDiDHon0lWcP+Qmt6UvZAAAAAAAAAacBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAF9eEAAAAAAAAAAAAAAAAAQQqLFQLwHyiH8LBbIsyTTUWmKKcnEi26xJVia/fd3KTtEQn+ZwcAonBDCzA1vRw+oHhAWKEJAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_string();

    // NFT vaa from ETH
    // let vaa_string ="AQAAAAABALt3I337ZUXILcnBsZqCMIG8TVwIePD/Gru2X50QwQMaZaFpI1XlnJ1LgsPhOTISd/YQXoTliIaZ/OVMPnp/7tgAZQuaBBt5AAAnEgAAAAAAAAAAAAAAAGoLUqwZjkhw5fN5fVtAODilu/2ZAAAAAAAAAAABAQAAAAAAAAAAAAAAACEVANGWC9t7ozkDR//YrUhriXoYJxIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADVZB9ilABQaXBmczovL2JhZnlyZWloMjczeDRqc2lkNjQ1YWE3NGE2d3RmYnlqNWNzbmNnazUzNGozeWUzcm00NHZpeWNwaXlhL21ldGFkYXRhLmpzb26+go39e1e9XPUt3DPsAFRBGQwfoFO6fZhM+R92rOMsEwAB".to_string();

    let vaa_bytes = decode(vaa_string).unwrap();

    let sig_start = 6;
    let num_signers = vaa_bytes[5] as usize;
    let sig_length = 66;

    let mut guardian_signatures = Vec::new();
    for i in 0..num_signers {
        let start = sig_start + i * sig_length;
        let mut signature = [0u8; 65];
        signature.copy_from_slice(&vaa_bytes[start + 1..start + 66]);
        guardian_signatures.push(wormhole_sdk::vaa::Signature {
            index: vaa_bytes[start],
            signature,
        });
    }

    let body = &vaa_bytes[sig_start + sig_length * num_signers..];

    let parsed_vaa = Vaa {
        version: vaa_bytes[0],
        guardian_set_index: u32::from_be_bytes(vaa_bytes[1..5].try_into().unwrap()),
        signatures: guardian_signatures,
        timestamp: u32::from_be_bytes(body[0..4].try_into().unwrap()),
        nonce: u32::from_be_bytes(body[4..8].try_into().unwrap()),
        emitter_chain: Chain::from(u16::from_be_bytes(body[8..10].try_into().unwrap())),
        emitter_address: Address(body[10..42].try_into().unwrap()),
        sequence: u64::from_be_bytes(body[42..50].try_into().unwrap()),
        consistency_level: body[50],
        payload: body[51..].to_vec(),
    };

    #[derive(Serialize, Deserialize, Debug)]
    enum MessageAlias {
        Transfer(Message),
        NftTransfer(NftMessage),
    }

    let payload = match serde_wormhole::from_slice(&parsed_vaa.payload) {
        Ok(message) => MessageAlias::Transfer(message),
        Err(_) => match serde_wormhole::from_slice(&parsed_vaa.payload) {
            Ok(nft_message) => MessageAlias::NftTransfer(nft_message),
            Err(_) => return Err(anyhow::anyhow!("Payload content not supported")),
        },
    };

    let payload_value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&payload)?)?;

    let inner_json = payload_value
        .get("NftTransfer")
        .or(payload_value.get("Transfer"))
        .ok_or_else(|| anyhow::anyhow!("Invalid payload"))?;

    // dbg!(&parsed_vaa);
    // dbg!(&inner_json);

    // let string = String::from_utf8(parsed_vaa.payload).unwrap();
    // println!("{}", string);
    // dbg!(&vaa_bytes);

    Ok(())
}
