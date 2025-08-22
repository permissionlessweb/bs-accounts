use cosmwasm_std::Uint256;
use std::str::FromStr;
use anyhow::{Result, Context, anyhow};


// When Uint256 Parsing Breaks Down
//
// The Problem: Uint256 can hold MUCH larger numbers than smaller integer types, so converting them often fails.
//
// Think of it like containers:
// - `u8` = small cup (holds 0-255)
// - `u16` = medium cup (holds 0-65,535) 
// - `u32` = large cup (holds 0-4.3 billion)
// - `u64` = huge cup (holds 0-18 quintillion)
// - `Uint256` = swimming pool (holds astronomically large numbers)
//
// When parsing breaks:
// ```rust
// // This works ✅
// let small = Uint256::from(100u64);
// let parsed: u64 = small.to_string().parse().unwrap(); // Success: 100
//
// // This breaks ❌
// let huge = Uint256::from(u64::MAX as u128 + 1); // Bigger than u64 can hold
// let parsed: u64 = huge.to_string().parse().unwrap(); // PANIC! Number too big
// ```
//
// The Rule: You can only parse a Uint256 into a smaller type if the number actually fits:
// - Values ≤ 255 (u8::MAX) → can parse to `u8`, `u16`, `u32`, `u64`, `u128`
// - Values ≤ 65,535 (u16::MAX) → can parse to `u16`, `u32`, `u64`, `u128` (but NOT `u8`)
// - Values ≤ 4.3 billion (u32::MAX) → can parse to `u32`, `u64`, `u128` (but NOT `u8` or `u16`)
// - Values ≤ 18.4 quintillion (u64::MAX) → can parse to `u64`, `u128` (but NOT `u8`, `u16`, or `u32`)
// - Values ≤ 340 undecillion (u128::MAX) → can parse to `u128` only (but NOT any smaller types)
// - Values > u128::MAX → CANNOT parse to any primitive integer type, only back to Uint256
//
// Safe approach:
// ```rust
// // Check first, then parse
// if uint256_val <= Uint256::from(u64::MAX) {
//     let safe: u64 = uint256_val.to_string().parse().unwrap();
// } else {
//     // Handle the "too big" case gracefully
//     return Err("Value too large for u64");
// }
// ```
//
// Bottom line: Always check if your Uint256 value fits in the target type before parsing, 
// or your code will panic when it encounters large numbers.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_tests() -> Result<()> {
        main()
    }
}

fn main() -> Result<()> {
    println!("=== Comprehensive Uint256 Parsing Tests ===\n");
    test_within_u64_range()?;
    test_within_u128_range()?;
    test_large_uint256_values()?;
    test_boundary_conditions()?;
    test_error_cases()?;
    test_parsing_approaches()?; 
    test_string_formats()?;
    test_uint256_to_primitives()?;
    test_safe_parsing_with_range_checks()?;
    println!("All tests completed successfully!");
    Ok(())
}

fn test_within_u64_range() -> Result<()> {
    println!("Test 1: Values within u64 range");
    
    let test_values = vec![0u64, 1, 100, 1000, u32::MAX as u64, u64::MAX];
    
    for val in test_values {
        let uint256_val = Uint256::from(val);
        let uint256_str = uint256_val.to_string();
        
        // Parse back to u64 - this SHOULD succeed for values in this test
        let parsed: u64 = uint256_str.parse()
            .with_context(|| format!("Failed to parse {} (originally {}) back to u64", uint256_str, val))?;
        
        if parsed != val {
            return Err(anyhow!("Round-trip failed: {} -> {} -> {}", val, uint256_str, parsed));
        }
        
        println!("  SUCCESS: {val} -> Uint256 -> String -> u64");
        
        // Also test parsing to other types - u128 should always work for u64 values
        let _u128_result: u128 = uint256_str.parse()
            .with_context(|| format!("u128 parsing should never fail for u64 values, but failed for {}", val))?;
        
        // u32 parsing should only work for values <= u32::MAX
        let u32_result: std::result::Result<u32, _> = uint256_str.parse();
        if val <= u32::MAX as u64 {
            u32_result.with_context(|| format!("u32 parsing should succeed for {} but failed", val))?;
        } else {
            if u32_result.is_ok() {
                return Err(anyhow!("u32 parsing should have failed for {} but succeeded", val));
            }
        }
    }
    println!();
    Ok(())
}

fn test_within_u128_range() -> Result<()> {
    println!("Test 2: Values within u128 range but exceeding u64");
    
    let test_values = vec![
        u64::MAX as u128 + 1,
        u64::MAX as u128 * 2,
        u128::MAX / 2,
        u128::MAX,
    ];
    
    for val in test_values {
        let uint256_val = Uint256::from(val);
        let uint256_str = uint256_val.to_string();
        
        // Should fail for u64
        let u64_result: std::result::Result<u64, _> = uint256_str.parse();
        if u64_result.is_ok() {
            return Err(anyhow!("u64 parsing should have failed for {} but succeeded", val));
        }
        
        // Should succeed for u128
        let parsed_u128: u128 = uint256_str.parse()
            .with_context(|| format!("u128 parsing should succeed for {} but failed", val))?;
        
        if parsed_u128 != val {
            return Err(anyhow!("u128 round-trip failed: {} -> {} -> {}", val, uint256_str, parsed_u128));
        }
        
        println!("  SUCCESS: {val} -> u128 parse worked, u64 correctly failed");
    }
    println!();
    Ok(())
}

fn test_large_uint256_values() -> Result<()> {
    println!("Test 3: Large Uint256 values exceeding u128");
    
    // Create some large Uint256 values
    let large_values = vec![
        Uint256::from_str("340282366920938463463374607431768211456")
            .map_err(|e| anyhow!("Failed to create large Uint256 value: {}", e))?, // u128::MAX + 1
        Uint256::from_str("1000000000000000000000000000000000000000")
            .map_err(|e| anyhow!("Failed to create very large Uint256 value: {}", e))?,
        Uint256::MAX,
    ];
    
    for val in large_values {
        let uint256_str = val.to_string();
        println!("  Testing large value: {}", &uint256_str[..50.min(uint256_str.len())]);
        
        // All smaller integer types should fail
        let u64_result: std::result::Result<u64, _> = uint256_str.parse();
        let u128_result: std::result::Result<u128, _> = uint256_str.parse();
        
        if u64_result.is_ok() {
            return Err(anyhow!("u64 parsing should have failed for large value but succeeded"));
        }
        
        if u128_result.is_ok() {
            return Err(anyhow!("u128 parsing should have failed for large value but succeeded"));
        }
        
        // But should be able to parse back to Uint256
        let parsed_uint256 = Uint256::from_str(&uint256_str)
            .map_err(|e| anyhow!("Failed to round-trip large Uint256 value: {}", e))?;
        
        if parsed_uint256 != val {
            return Err(anyhow!("Uint256 round-trip failed for large value"));
        }
        
        println!("    SUCCESS: Uint256 round-trip worked, smaller types correctly failed");
    }
    println!();
    Ok(())
}

fn test_boundary_conditions() -> Result<()> {
    println!("Test 4: Boundary conditions");
    
    // Test exact boundary values
    let boundaries = vec![
        ("u8::MAX", Uint256::from(u8::MAX), u8::MAX as u128),
        ("u16::MAX", Uint256::from(u16::MAX), u16::MAX as u128),
        ("u32::MAX", Uint256::from(u32::MAX), u32::MAX as u128),
        ("u64::MAX", Uint256::from(u64::MAX), u64::MAX as u128),
        ("u128::MAX", Uint256::from(u128::MAX), u128::MAX),
        ("u8::MAX + 1", Uint256::from(u8::MAX as u16 + 1), u8::MAX as u128 + 1),
        ("u16::MAX + 1", Uint256::from(u16::MAX as u32 + 1), u16::MAX as u128 + 1),
        ("u32::MAX + 1", Uint256::from(u32::MAX as u64 + 1), u32::MAX as u128 + 1),
        ("u64::MAX + 1", Uint256::from(u64::MAX as u128 + 1), u64::MAX as u128 + 1),
    ];
    
    for (name, val, expected_num) in boundaries {
        let uint256_str = val.to_string();
        println!("  Testing {name}: {}", uint256_str);
        
        // Test parsing to different integer types with expected outcomes
        let u8_result: std::result::Result<u8, _> = uint256_str.parse();
        let u16_result: std::result::Result<u16, _> = uint256_str.parse();
        let u32_result: std::result::Result<u32, _> = uint256_str.parse();
        let u64_result: std::result::Result<u64, _> = uint256_str.parse();
        let u128_result: std::result::Result<u128, _> = uint256_str.parse();
        
        // Verify expected outcomes
        if expected_num <= u8::MAX as u128 {
            u8_result.with_context(|| format!("u8 should succeed for {name} but failed"))?;
        } else if u8_result.is_ok() {
            return Err(anyhow!("u8 should fail for {name} but succeeded"));
        }
        
        if expected_num <= u16::MAX as u128 {
            u16_result.with_context(|| format!("u16 should succeed for {name} but failed"))?;
        } else if u16_result.is_ok() {
            return Err(anyhow!("u16 should fail for {name} but succeeded"));
        }
        
        if expected_num <= u32::MAX as u128 {
            u32_result.with_context(|| format!("u32 should succeed for {name} but failed"))?;
        } else if u32_result.is_ok() {
            return Err(anyhow!("u32 should fail for {name} but succeeded"));
        }
        
        if expected_num <= u64::MAX as u128 {
            u64_result.with_context(|| format!("u64 should succeed for {name} but failed"))?;
        } else if u64_result.is_ok() {
            return Err(anyhow!("u64 should fail for {name} but succeeded"));
        }
        
        // u128 should always work for values we can create from u128::MAX or below
        let parsed_u128 = u128_result.with_context(|| format!("u128 should succeed for {name} but failed"))?;
        if parsed_u128 != expected_num {
            return Err(anyhow!("u128 value mismatch for {name}: expected {}, got {}", expected_num, parsed_u128));
        }
        
        println!("    SUCCESS: All boundary checks passed");
    }
    println!();
    Ok(())
}

fn test_error_cases() -> Result<()> {
    println!("Test 5: Error handling and validation");
    
    // Test invalid string formats - these SHOULD fail
    let invalid_strings = vec![
        "",
        " ",
        "abc",
        "123abc",
        "-123",
        "123.45",
        "0x123",
        "  123  ", // Leading/trailing whitespace might work depending on implementation
    ];
    
    for invalid_str in invalid_strings {
        println!("  Testing invalid string: '{invalid_str}'");
        
        let u64_result: std::result::Result<u64, _> = invalid_str.parse();
        let uint256_result = Uint256::from_str(invalid_str);
        
        // Most of these should fail - whitespace might be handled differently
        if invalid_str.trim().chars().all(|c| c.is_ascii_digit()) && !invalid_str.trim().is_empty() {
            // This might actually succeed (whitespace trimmed)
            println!("    Note: '{}' might succeed after trimming", invalid_str);
        } else {
            // These should definitely fail
            if u64_result.is_ok() {
                return Err(anyhow!("u64 parsing should have failed for '{}' but succeeded", invalid_str));
            }
            if uint256_result.is_ok() {
                return Err(anyhow!("Uint256 parsing should have failed for '{}' but succeeded", invalid_str));
            }
            println!("    SUCCESS: Correctly rejected invalid input");
        }
    }
    println!();
    Ok(())
}

fn test_parsing_approaches() -> Result<()> {
    println!("Test 6: Different parsing approaches");
    
    let test_value = Uint256::from(12345u64);
    let uint256_str = test_value.to_string();
    
    println!("  Original Uint256: {test_value}");
    println!("  String representation: '{uint256_str}'");
    
    // Method 1: Direct parse
    let method1_result: u64 = uint256_str.parse()
        .context("Method 1 (direct parse) should succeed for small value")?;
    
    // Method 2: FromStr trait
    let method2_result: u64 = u64::from_str(&uint256_str)
        .context("Method 2 (FromStr trait) should succeed for small value")?;
    
    // Method 3: Uint256 from_str then manual validation
    let uint256_parsed = Uint256::from_str(&uint256_str)
        .map_err(|e| anyhow!("Method 3 (Uint256 from_str) should succeed: {}", e))?;
    
    // Verify all methods produce the same result
    if method1_result != method2_result {
        return Err(anyhow!("Method 1 and 2 produced different results: {} vs {}", method1_result, method2_result));
    }
    
    if method1_result != 12345 {
        return Err(anyhow!("Parsing result {} doesn't match expected 12345", method1_result));
    }
    
    if uint256_parsed != test_value {
        return Err(anyhow!("Uint256 round-trip failed"));
    }
    
    println!("  SUCCESS: All parsing methods produced correct results");
    println!();
    Ok(())
}

fn test_string_formats() -> Result<()> {
    println!("Test 7: String format validation");
    
    let test_value = Uint256::from(123456789u64);
    let uint256_str = test_value.to_string();
    
    println!("  Uint256 string format: '{uint256_str}'");
    println!("  String length: {}", uint256_str.len());
    
    // Verify string format properties
    if !uint256_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("Uint256 string should contain only digits but contains non-digits"));
    }
    
    if uint256_str.starts_with('0') && uint256_str.len() > 1 {
        return Err(anyhow!("Uint256 string should not have leading zeros (except for zero itself)"));
    }
    
    if uint256_str.is_empty() {
        return Err(anyhow!("Uint256 string should not be empty"));
    }
    
    // Test zero representation
    let zero_uint256 = Uint256::zero();
    let zero_str = zero_uint256.to_string();
    if zero_str != "0" {
        return Err(anyhow!("Zero should be represented as '0' but got '{}'", zero_str));
    }
    
    // Test max value string properties
    let max_str = Uint256::MAX.to_string();
    if max_str.is_empty() {
        return Err(anyhow!("Max Uint256 string should not be empty"));
    }
    
    if !max_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("Max Uint256 string should contain only digits"));
    }
    
    println!("  Max Uint256 string length: {}", max_str.len());
    println!("  Max Uint256 first 20 chars: {}", &max_str[..20.min(max_str.len())]);
    
    println!("  SUCCESS: All string format validations passed");
    println!();
    Ok(())
}

// Helper function to demonstrate safe parsing with proper error handling
fn safe_parsing_with_range_check(uint256_val: Uint256) -> Result<u64> {
    let str_val = uint256_val.to_string();
    
    // First check if the value is within u64 range
    if uint256_val > Uint256::from(u64::MAX) {
        return Err(anyhow!("Value {} exceeds u64::MAX", str_val));
    }
    
    // Safe to parse since we've verified the range
    str_val.parse::<u64>()
        .with_context(|| format!("Failed to parse {} to u64 despite range check", str_val))
}

fn test_uint256_to_primitives() -> Result<()> {
    println!("Test 8: Uint256 to primitive type conversions");
    
    // Test various Uint256 values and their conversion to all primitive types
    let test_cases = vec![
        ("Small value", Uint256::from(42u8)),
        ("u8::MAX", Uint256::from(u8::MAX)),
        ("u8::MAX + 1", Uint256::from(u8::MAX as u16 + 1)),
        ("u16::MAX", Uint256::from(u16::MAX)),
        ("u16::MAX + 1", Uint256::from(u16::MAX as u32 + 1)),
        ("u32::MAX", Uint256::from(u32::MAX)),
        ("u32::MAX + 1", Uint256::from(u32::MAX as u64 + 1)),
        ("u64::MAX", Uint256::from(u64::MAX)),
        ("u64::MAX + 1", Uint256::from(u64::MAX as u128 + 1)),
        ("u128::MAX", Uint256::from(u128::MAX)),
        ("Large Uint256", Uint256::from_str("340282366920938463463374607431768211456").unwrap()),
    ];
    
    for (name, uint256_val) in test_cases {
        println!("  Testing {}: {}", name, uint256_val);
        let uint256_str = uint256_val.to_string();
        
        // Test conversion to each primitive type
        test_conversion_to_u8(&uint256_str, &uint256_val, name)?;
        test_conversion_to_u16(&uint256_str, &uint256_val, name)?;
        test_conversion_to_u32(&uint256_str, &uint256_val, name)?;
        test_conversion_to_u64(&uint256_str, &uint256_val, name)?;
        test_conversion_to_u128(&uint256_str, &uint256_val, name)?;
        
        println!("    ✓ All conversions behaved as expected");
    }
    println!();
    Ok(())
}

fn test_conversion_to_u8(uint256_str: &str, uint256_val: &Uint256, test_name: &str) -> Result<()> {
    let result: std::result::Result<u8, _> = uint256_str.parse();
    let should_succeed = *uint256_val <= Uint256::from(u8::MAX);
    
    match (result.is_ok(), should_succeed) {
        (true, true) => {
            let parsed = result.unwrap();
            if Uint256::from(parsed) != *uint256_val {
                return Err(anyhow!("u8 conversion for {} produced wrong value: {} != {}", test_name, parsed, uint256_val));
            }
        },
        (false, false) => {
            // Expected failure - this is correct
        },
        (true, false) => {
            return Err(anyhow!("u8 conversion for {} should have failed but succeeded with: {}", test_name, result.unwrap()));
        },
        (false, true) => {
            return Err(anyhow!("u8 conversion for {} should have succeeded but failed with: {}", test_name, result.unwrap_err()));
        }
    }
    Ok(())
}

fn test_safe_parsing_with_range_checks() -> Result<()> {
    println!("Test 9: Safe parsing with range checks");
    
    let test_cases = vec![
        ("Small value", Uint256::from(1000u64), true),
        ("Medium value", Uint256::from(u32::MAX as u64), true),
        ("u64::MAX", Uint256::from(u64::MAX), true),
        ("u64::MAX + 1", Uint256::from(u64::MAX as u128 + 1), false),
        ("u128::MAX", Uint256::from(u128::MAX), false),
        ("Large Uint256", Uint256::from_str("999999999999999999999999999999999999999").unwrap(), false),
    ];
    
    for (name, uint256_val, should_succeed) in test_cases {
        println!("  Testing safe parsing for {}: {}", name, uint256_val);
        
        let result = safe_parsing_with_range_check(uint256_val.clone());
        
        match (result.is_ok(), should_succeed) {
            (true, true) => {
                let parsed_u64 = result.unwrap();
                // Verify round-trip accuracy
                if Uint256::from(parsed_u64) != uint256_val {
                    return Err(anyhow!("Safe parsing for {} produced incorrect value: {} != {}", name, parsed_u64, uint256_val));
                }
                println!("    ✓ SUCCESS: Safely parsed to u64: {}", parsed_u64);
            },
            (false, false) => {
                println!("    ✓ SUCCESS: Correctly rejected ({})", result.unwrap_err());
            },
            (true, false) => {
                return Err(anyhow!("Safe parsing for {} should have failed but succeeded with: {}", name, result.unwrap()));
            },
            (false, true) => {
                return Err(anyhow!("Safe parsing for {} should have succeeded but failed with: {}", name, result.unwrap_err()));
            }
        }
    }
    
    // Test the safe parsing approach vs unsafe parsing
    println!("  Demonstrating safe vs unsafe parsing:");
    
    let dangerous_values = vec![
        Uint256::from(u64::MAX as u128 + 1),
        Uint256::MAX,
    ];
    
    for val in dangerous_values {
        let val_str = val.to_string();
        println!("    Testing dangerous value: {}", &val_str[..50.min(val_str.len())]);
        
        // Unsafe approach (would panic with .expect())
        let unsafe_result: std::result::Result<u64, _> = val_str.parse();
        if unsafe_result.is_ok() {
            return Err(anyhow!("Unsafe parsing should have failed for large value"));
        }
        println!("      Unsafe .parse() correctly failed: {}", unsafe_result.unwrap_err());
        
        // Safe approach with range check
        let safe_result = safe_parsing_with_range_check(val);
        if safe_result.is_ok() {
            return Err(anyhow!("Safe parsing should have failed for large value"));
        }
        println!("      Safe range check correctly failed: {}", safe_result.unwrap_err());
    }
    
    println!();
    Ok(())
}

fn test_conversion_to_u16(uint256_str: &str, uint256_val: &Uint256, test_name: &str) -> Result<()> {
    let result: std::result::Result<u16, _> = uint256_str.parse();
    let should_succeed = *uint256_val <= Uint256::from(u16::MAX);
    
    match (result.is_ok(), should_succeed) {
        (true, true) => {
            let parsed = result.unwrap();
            if Uint256::from(parsed) != *uint256_val {
                return Err(anyhow!("u16 conversion for {} produced wrong value: {} != {}", test_name, parsed, uint256_val));
            }
        },
        (false, false) => {
            // Expected failure - this is correct
        },
        (true, false) => {
            return Err(anyhow!("u16 conversion for {} should have failed but succeeded with: {}", test_name, result.unwrap()));
        },
        (false, true) => {
            return Err(anyhow!("u16 conversion for {} should have succeeded but failed with: {}", test_name, result.unwrap_err()));
        }
    }
    Ok(())
}

fn test_conversion_to_u32(uint256_str: &str, uint256_val: &Uint256, test_name: &str) -> Result<()> {
    let result: std::result::Result<u32, _> = uint256_str.parse();
    let should_succeed = *uint256_val <= Uint256::from(u32::MAX);
    
    match (result.is_ok(), should_succeed) {
        (true, true) => {
            let parsed = result.unwrap();
            if Uint256::from(parsed) != *uint256_val {
                return Err(anyhow!("u32 conversion for {} produced wrong value: {} != {}", test_name, parsed, uint256_val));
            }
        },
        (false, false) => {
            // Expected failure - this is correct
        },
        (true, false) => {
            return Err(anyhow!("u32 conversion for {} should have failed but succeeded with: {}", test_name, result.unwrap()));
        },
        (false, true) => {
            return Err(anyhow!("u32 conversion for {} should have succeeded but failed with: {}", test_name, result.unwrap_err()));
        }
    }
    Ok(())
}

fn test_conversion_to_u64(uint256_str: &str, uint256_val: &Uint256, test_name: &str) -> Result<()> {
    let result: std::result::Result<u64, _> = uint256_str.parse();
    let should_succeed = *uint256_val <= Uint256::from(u64::MAX);
    
    match (result.is_ok(), should_succeed) {
        (true, true) => {
            let parsed = result.unwrap();
            if Uint256::from(parsed) != *uint256_val {
                return Err(anyhow!("u64 conversion for {} produced wrong value: {} != {}", test_name, parsed, uint256_val));
            }
        },
        (false, false) => {
            // Expected failure - this is correct
        },
        (true, false) => {
            return Err(anyhow!("u64 conversion for {} should have failed but succeeded with: {}", test_name, result.unwrap()));
        },
        (false, true) => {
            return Err(anyhow!("u64 conversion for {} should have succeeded but failed with: {}", test_name, result.unwrap_err()));
        }
    }
    Ok(())
}

fn test_conversion_to_u128(uint256_str: &str, uint256_val: &Uint256, test_name: &str) -> Result<()> {
    let result: std::result::Result<u128, _> = uint256_str.parse();
    let should_succeed = *uint256_val <= Uint256::from(u128::MAX);
    
    match (result.is_ok(), should_succeed) {
        (true, true) => {
            let parsed = result.unwrap();
            if Uint256::from(parsed) != *uint256_val {
                return Err(anyhow!("u128 conversion for {} produced wrong value: {} != {}", test_name, parsed, uint256_val));
            }
        },
        (false, false) => {
            // Expected failure - this is correct
        },
        (true, false) => {
            return Err(anyhow!("u128 conversion for {} should have failed but succeeded with: {}", test_name, result.unwrap()));
        },
        (false, true) => {
            return Err(anyhow!("u128 conversion for {} should have succeeded but failed with: {}", test_name, result.unwrap_err()));
        }
    }
    Ok(())
}