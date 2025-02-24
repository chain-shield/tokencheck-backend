//! Utilities for type conversion and formatting.
//!
//! This module provides functions for converting between various types used in the Ethereum
//! ecosystem (e.g., U256, H256, Address) and other common representations such as strings,
//! bytes arrays, and decimal values. Additionally, it includes helper functions for truncating
//! Unicode strings and computing function selectors.

use anyhow::{anyhow, Context, Result};
use ethers::abi::{Address, Bytes};
use ethers::utils::{format_units, keccak256};
use ethers::{
    core::types::{transaction::eip2718::TypedTransaction, H256, I256, U256},
    types::{Eip1559TransactionRequest, NameOrAddress, Transaction, TransactionRequest},
};
use rust_decimal::Decimal;
use std::convert::TryInto;
use std::str::FromStr;

/// Truncates `text` to at most `max_len` Unicode characters.
///
/// This function ensures that the text is not cut in the middle of a multibyte character by
/// operating on Unicode scalar values.
///
/// # Arguments
///
/// * `text` - The input string to truncate.
/// * `max_len` - The maximum number of characters to include.
///
/// # Returns
///
/// A new `String` containing at most `max_len` characters.
pub fn truncate_code_unicode(text: &str, max_len: usize) -> String {
    let mut chars = text.chars();
    if text.chars().count() > max_len {
        // Take the first `max_len` chars and collect into a new String.
        chars.by_ref().take(max_len).collect()
    } else {
        text.to_string()
    }
}

/// Computes the function selector for a given function signature.
///
/// The function selector is derived as the first 4 bytes of the Keccak256 hash of the provided
/// function signature.
///
/// # Arguments
///
/// * `function_signature` - A string slice representing the signature of the function.
///
/// # Returns
///
/// A `Bytes` instance containing the 4-byte function selector.
pub fn get_function_selector(function_signature: &str) -> Bytes {
    let hash = H256::from(keccak256(function_signature.as_bytes()));
    Bytes::from(&hash[0..4])
}

/// Formats a U256 amount into a string representation with 5 decimal places.
///
/// The function formats the amount using the specified `decimals`, converts it to a `Decimal`,
/// rounds it to 5 decimal places, and then returns it as a `String`.
///
/// # Arguments
///
/// * `amount` - The U256 value to format.
/// * `decimals` - The number of decimal places to account for during formatting.
///
/// # Returns
///
/// A `String` representing the formatted number.
pub fn format_to_5_decimals_decimal(amount: U256, decimals: u32) -> String {
    let amount_str = format_units(amount, decimals).unwrap_or_default();
    // Convert the formatted string to a Decimal
    let decimal_val = Decimal::from_str(&amount_str).unwrap_or_default();
    // Round to 5 decimal places
    let truncated_val = decimal_val.round_dp(5);
    // Convert back to a string
    truncated_val.to_string()
}

/// Converts a legacy `Transaction` into an EIP2718 `TypedTransaction`.
///
/// Depending on the transaction type, this function creates either an EIP-1559 transaction or a
/// legacy transaction. If the `to` field is missing in the original transaction, an error is returned.
///
/// # Arguments
///
/// * `tx` - A reference to a `Transaction`.
///
/// # Errors
///
/// Returns an error if the `to` field is not present in the provided transaction.
///
/// # Returns
///
/// A `TypedTransaction` wrapped in an `anyhow::Result`.
pub fn convert_transaction_to_typed_transaction(
    tx: &Transaction,
) -> anyhow::Result<TypedTransaction> {
    // Ensure the `to` address is present.
    let to = tx
        .to
        .clone()
        .ok_or_else(|| anyhow!("Missing 'to' field in transaction"))?;
    let to = NameOrAddress::Address(to);

    if tx.transaction_type == Some(2.into()) {
        // EIP-1559 transaction.
        Ok(TypedTransaction::Eip1559(Eip1559TransactionRequest {
            from: Some(tx.from),
            to: Some(to),
            value: Some(tx.value),
            data: Some(tx.input.clone()),
            nonce: Some(tx.nonce),
            gas: Some(tx.gas),
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            access_list: tx.access_list.clone().unwrap_or_default(),
            chain_id: tx.chain_id.map(|id| ethers::types::U64::from(id.as_u64())),
        }))
    } else {
        // Legacy transaction.
        Ok(TypedTransaction::Legacy(TransactionRequest {
            from: Some(tx.from),
            to: Some(to),
            value: Some(tx.value),
            data: Some(tx.input.clone()),
            nonce: Some(tx.nonce),
            gas: Some(tx.gas),
            gas_price: tx.gas_price,
            chain_id: tx.chain_id.map(|id| ethers::types::U64::from(id.as_u64())),
        }))
    }
}

/// Converts a U256 value to an f64, taking into account the specified number of decimals.
///
/// The process involves first formatting the U256 value into a decimal string using the provided
/// number of decimals, and then parsing this string into an f64.
///
/// # Arguments
///
/// * `value` - The U256 value to convert.
/// * `decimals` - The number of decimal places associated with the U256 value.
///
/// # Errors
///
/// Returns an error if the conversion from U256 to a string or the parsing of the string into an f64 fails.
pub fn u256_to_f64_with_decimals(value: U256, decimals: u32) -> anyhow::Result<f64> {
    // Convert U256 to a string with respect to the given decimals.
    let value_str = format_units(value, decimals)?;
    // Parse the decimal string into an f64.
    let value_f64 = f64::from_str(&value_str)?;
    Ok(value_f64)
}

/// Generates an H256 hash from the provided string.
///
/// The hash is computed using the Keccak256 algorithm on the UTF-8 bytes of the input string.
///
/// # Arguments
///
/// * `str` - The input string.
///
/// # Returns
///
/// An `H256` hash.
pub fn str_to_h256_hash(str: &str) -> H256 {
    H256::from(keccak256(str.as_bytes()))
}

/// Converts a U256 number into a 32-byte array in big-endian order.
///
/// # Arguments
///
/// * `number` - The U256 value to convert.
///
/// # Returns
///
/// A [u8; 32] array representing the U256 value.
pub fn u256_to_bytes_array(number: U256) -> [u8; 32] {
    let mut number_bytes = [0u8; 32];
    number.to_big_endian(&mut number_bytes);
    number_bytes
}

/// Converts a boolean value into a 32-byte array.
///
/// The last byte of the array is set to 1 if the boolean is true, or 0 if false.
///
/// # Arguments
///
/// * `boolean` - The boolean value to convert.
///
/// # Returns
///
/// A [u8; 32] array representing the boolean.
pub fn boolean_to_bytes_array(boolean: bool) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[31] = if boolean { 1 } else { 0 };
    bytes
}

/// Converts a u8 value into a 32-byte array.
///
/// The value is placed in the last byte of the array.
///
/// # Arguments
///
/// * `value` - The u8 value to convert.
///
/// # Returns
///
/// A [u8; 32] array representing the u8 value.
pub fn u8_to_bytes_array(value: u8) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[31] = value;
    bytes
}

/// Converts a u16 value into a 32-byte array.
///
/// The u16 is stored in the last two bytes of the array in big-endian order.
///
/// # Arguments
///
/// * `value` - The u16 value to convert.
///
/// # Returns
///
/// A [u8; 32] array representing the u16 value.
pub fn u16_to_bytes_array(value: u16) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[30..32].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Converts an Ethereum `Address` into a 32-byte array.
///
/// The address bytes are copied into the last 20 bytes of the returned array.
///
/// # Arguments
///
/// * `address` - The Ethereum address.
///
/// # Returns
///
/// A [u8; 32] array representing the address.
pub fn address_to_bytes_array(address: Address) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[12..32].copy_from_slice(address.as_bytes());
    bytes
}

/// Returns a string representation of an Ethereum `Address`.
///
/// The address is formatted using the `Debug` formatter.
///
/// # Arguments
///
/// * `address` - The Ethereum address to format.
///
/// # Returns
///
/// A `String` representing the address.
pub fn address_to_string(address: Address) -> String {
    format!("{:?}", address)
}

/// Parses a string into a boolean value.
///
/// Accepts case-insensitive "true" or "false". Returns an error for any other value.
///
/// # Arguments
///
/// * `s` - The input string representing a boolean.
///
/// # Errors
///
/// Returns an error if the input string is not "true" or "false".
///
/// # Returns
///
/// A boolean value corresponding to the input string.
pub fn string_to_bool(s: &str) -> anyhow::Result<bool> {
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow!("Invalid boolean string")),
    }
}

/// Converts an f64 value into a U256 by scaling it by 1e18.
///
/// The function checks for invalid inputs (NaN, infinity, or negative values) and ensures
/// that the scaled value is within the bounds of a u128 before conversion.
///
/// # Arguments
///
/// * `value` - The f64 value to convert.
///
/// # Errors
///
/// Returns an error if the f64 value is NaN, infinite, negative, or too large to be represented as a U256.
pub fn f64_to_u256(value: f64) -> Result<U256> {
    if value.is_nan() || value.is_infinite() {
        return Err(anyhow!("Invalid f64 value: NaN or Infinity"));
    }

    if value < 0.0 {
        return Err(anyhow!("Negative values are not supported"));
    }

    // Scale the float by 1e18 to move the decimal point.
    let scaled_value = value * 1e18;

    // Check for overflow/underflow issues before conversion.
    if scaled_value > u128::MAX as f64 {
        return Err(anyhow!("Value is too large for u128"));
    }

    // Safely convert the scaled float to u128.
    let integer_value = scaled_value as u128;

    // Convert u128 to U256.
    Ok(U256::from(integer_value))
}

/// Attempts to convert a U256 value to an f64.
///
/// This conversion is performed by first attempting to convert the U256 into a u128. If the conversion
/// is successful, the resulting u128 is cast to an f64; otherwise, the function returns `None`.
///
/// # Arguments
///
/// * `value` - The U256 value to convert.
///
/// # Returns
///
/// An `Option<f64>` containing the f64 representation, or `None` if the U256 is too large.
pub fn u256_to_f64(value: U256) -> Option<f64> {
    // Attempt to convert U256 to u128.
    if let Ok(val) = value.try_into() {
        let val_u128: u128 = val;
        // Convert u128 to f64; note that precision may be lost.
        Some(val_u128 as f64)
    } else {
        // The U256 value was too large to fit into a u128.
        None
    }
}

/// Converts an I256 value to an f64 if it fits within f64 precision limits.
///
/// The conversion is considered safe if the I256 value is within \(2^{53}\) (i.e., within the safe
/// integer range of f64). It first attempts to cast the I256 to an i64 and then converts it to f64.
///
/// # Arguments
///
/// * `value` - The I256 value to convert.
///
/// # Errors
///
/// Returns an error if the I256 value is out of the representable range for f64 or if the conversion to i64 fails.
pub fn i256_to_f64(value: I256) -> Result<f64, &'static str> {
    // Check if the value fits within safe f64 integer precision.
    if value < I256::from(2_i128.pow(53)) && value > I256::from(-2_i128.pow(53)) {
        // Safe to convert to i64 then to f64.
        let value_i64 =
            i64::try_from(value).map_err(|_| "Conversion to i64 failed due to overflow")?;
        Ok(value_i64 as f64)
    } else {
        Err("Value out of f64 precision range")
    }
}

/// Converts an H256 hash into an Ethereum `Address`.
///
/// The address is derived by taking the last 20 bytes of the H256 hash.
///
/// # Arguments
///
/// * `h` - A reference to an H256 hash.
///
/// # Returns
///
/// An Ethereum `Address`.
pub fn h256_to_address(h: &H256) -> Address {
    let bytes = h.as_bytes();
    // Addresses are contained in the last 20 bytes of the H256.
    Address::from_slice(&bytes[12..32])
}

/// Converts an H256 hash into a U256 number.
///
/// The conversion interprets the H256 bytes in big-endian order.
///
/// # Arguments
///
/// * `h` - A reference to an H256 hash.
///
/// # Returns
///
/// A U256 value representing the hash.
pub fn h256_to_u256(h: &H256) -> U256 {
    U256::from_big_endian(h.as_bytes())
}
