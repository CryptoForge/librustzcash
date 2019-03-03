use bech32::{convert_bits, Bech32};
use failure::{format_err, Error};
use pairing::bls12_381::Bls12;
use sapling_crypto::{
    jubjub::edwards,
    primitives::{Diversifier, PaymentAddress},
};
use std::io::{self, Write};
use zcash_primitives::JUBJUB;
use zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};

fn bech32_encode<F>(hrp: &str, write: F) -> String
where
    F: Fn(&mut dyn Write) -> io::Result<()>,
{
    let mut data: Vec<u8> = vec![];
    write(&mut data).expect("Should be able to write to a Vec");

    let converted =
        convert_bits(&data, 8, 5, true).expect("Should be able to convert Vec<u8> to Vec<u5>");
    let encoded = Bech32::new_check_data(hrp.into(), converted).expect("hrp is not empty");

    encoded.to_string()
}

fn bech32_decode<T, F>(hrp: &str, s: &str, read: F) -> Result<T, Error>
where
    F: Fn(Vec<u8>) -> Result<T, Error>,
{
    let encoded = Bech32::from_str_lenient(s)?;
    if encoded.hrp() == hrp {
        let data = convert_bits(encoded.data(), 5, 8, false)?;
        read(data)
    } else {
        Err(format_err!("Invalid HRP"))
    }
}

pub fn encode_extended_spending_key(hrp: &str, extsk: &ExtendedSpendingKey) -> String {
    bech32_encode(hrp, |w| extsk.write(w))
}

pub fn decode_extended_spending_key(hrp: &str, s: &str) -> Result<ExtendedSpendingKey, Error> {
    bech32_decode(hrp, s, |data| Ok(ExtendedSpendingKey::read(&data[..])?))
}

pub fn encode_extended_full_viewing_key(hrp: &str, extfvk: &ExtendedFullViewingKey) -> String {
    bech32_encode(hrp, |w| extfvk.write(w))
}

pub fn decode_extended_full_viewing_key(
    hrp: &str,
    s: &str,
) -> Result<ExtendedFullViewingKey, Error> {
    bech32_decode(hrp, s, |data| Ok(ExtendedFullViewingKey::read(&data[..])?))
}

pub fn encode_payment_address(hrp: &str, addr: &PaymentAddress<Bls12>) -> String {
    bech32_encode(hrp, |w| {
        w.write_all(&addr.diversifier.0)?;
        addr.pk_d.write(w)
    })
}

pub fn decode_payment_address(hrp: &str, s: &str) -> Result<PaymentAddress<Bls12>, Error> {
    bech32_decode(hrp, s, |data| {
        let mut diversifier = Diversifier([0; 11]);
        diversifier.0.copy_from_slice(&data[0..11]);
        match edwards::Point::<Bls12, _>::read(&data[11..], &JUBJUB) {
            Ok(p) => match p.as_prime_order(&JUBJUB) {
                Some(pk_d) => Ok(PaymentAddress { pk_d, diversifier }),
                None => Err(format_err!("pk_d is not prime order")),
            },
            Err(e) => Err(format_err!("{}", e)),
        }
    })
}

#[cfg(test)]
mod tests {
    use pairing::bls12_381::Bls12;
    use rand::{SeedableRng, XorShiftRng};
    use sapling_crypto::{
        jubjub::edwards,
        primitives::{Diversifier, PaymentAddress},
    };
    use zcash_primitives::JUBJUB;

    use super::{decode_payment_address, encode_payment_address};
    use crate::constants;

    #[test]
    fn payment_address() {
        let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

        let addr = PaymentAddress {
            diversifier: Diversifier([0u8; 11]),
            pk_d: edwards::Point::<Bls12, _>::rand(rng, &JUBJUB).mul_by_cofactor(&JUBJUB),
        };

        let encoded_main =
            "zs1qqqqqqqqqqqqqqqqqqxrrfaccydp867g6zg7ne5ht37z38jtfyw0ygmp0ja6hhf07twjqj2ug6x";
        let encoded_test =
            "ztestsapling1qqqqqqqqqqqqqqqqqqxrrfaccydp867g6zg7ne5ht37z38jtfyw0ygmp0ja6hhf07twjq6awtaj";

        assert_eq!(
            encode_payment_address(constants::HRP_SAPLING_PAYMENT_ADDRESS_MAIN, &addr),
            encoded_main
        );
        assert_eq!(
            decode_payment_address(constants::HRP_SAPLING_PAYMENT_ADDRESS_MAIN, encoded_main)
                .unwrap(),
            addr
        );

        assert_eq!(
            encode_payment_address(constants::HRP_SAPLING_PAYMENT_ADDRESS_TEST, &addr),
            encoded_test
        );
        assert_eq!(
            decode_payment_address(constants::HRP_SAPLING_PAYMENT_ADDRESS_TEST, encoded_test)
                .unwrap(),
            addr
        );
    }
}
