use crate::prelude::*;
use base64::decode;
use wormhole_sdk::{Address, Chain, Vaa};

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
}

async fn run(mut ctx: Context, input: Input) -> Result<Output, CommandError> {
    let vaa_string = &input.vaa;

    let vaa_bytes = decode(&vaa_string)
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

    let parsed_vaa: Vaa<Vec<u8>> = Vaa {
        version: vaa_bytes[0],
        guardian_set_index: u32::from_be_bytes(
            vaa_bytes[1..5]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert guardian_set_index"))?,
        ),
        signatures: guardian_signatures,
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

    Ok(Output { parsed_vaa })
}

#[test]
fn test() {
    let vaa_string:String = "AQAAAAABAE9eT/T0B917C5+ZQEHdlDUD/b7PNfTkyy/mXX7LPSJzVS6VTJx1gigK7xCic3UywM5/ehtUnZ/HCdoLQtOLX1IBZLYUVg1YsBoAAcARZHHBCI3jyzPKm9l0vBFJ3DJ4Yh+vmP6ZmTrfVHxrAAAAAAAAAAABSGVsbG8gV29ybGQh".to_string();
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

    dbg!(&parsed_vaa);
    // let string = String::from_utf8(parsed_vaa.payload).unwrap();
    // println!("{}", string);
}
