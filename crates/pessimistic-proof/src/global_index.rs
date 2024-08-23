use reth_primitives::U256;
use serde::{Deserialize, Serialize};

/// The [`GlobalIndex`] uniquely references one leaf within one Global Exit Tree.
/// Further defined by the LXLY specifications.
/// | 191 bits |    1 bit      |    32 bits   |    32 bits   |
/// |    0     |  mainnet flag | rollup index |  leaf index  |
#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default, PartialEq)]
pub struct GlobalIndex {
    pub mainnet_flag: bool,
    pub rollup_index: u32,
    pub leaf_index: u32,
}

impl GlobalIndex {
    const MAINNET_FLAG_OFFSET: usize = 2 * 32;
}

impl From<U256> for GlobalIndex {
    fn from(value: U256) -> Self {
        let bytes = value.as_le_slice();

        let mainnet_flag = value.bit(Self::MAINNET_FLAG_OFFSET);
        let rollup_index = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let leaf_index = u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        Self {
            mainnet_flag,
            rollup_index,
            leaf_index,
        }
    }
}

impl GlobalIndex {
    #[allow(unused)]
    fn to_u256(self) -> U256 {
        let mut bytes = [0u8; 32];

        let leaf_bytes = self.leaf_index.to_le_bytes();
        bytes[0..4].copy_from_slice(&leaf_bytes);

        let rollup_bytes = self.rollup_index.to_le_bytes();
        bytes[4..8].copy_from_slice(&rollup_bytes);

        if self.mainnet_flag {
            bytes[8] |= 0x01;
        }

        U256::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(raw: &str, expected: GlobalIndex) {
        let global_index_u256 = U256::from_str_radix(raw, 10).unwrap();
        assert_eq!(global_index_u256, GlobalIndex::from(global_index_u256).to_u256());
        assert_eq!(expected, GlobalIndex::from(global_index_u256));
    }

    #[test]
    fn test_global_index() {
        // https://bridge-api.zkevm-g-mainnet.com/bridges/0xa1D5E9CB4f6a09fcF8b938435b0DE63270C67537
        check(
            "18446744073709748107",
            GlobalIndex {
                mainnet_flag: true,
                rollup_index: 0,
                leaf_index: 196491,
            },
        );

        // https://etherscan.io/tx/0xd9bc7b7de2df86e08221e41806cfa798693d700f1f644810beb0e7c14706b82d
        check(
            "4294968029",
            GlobalIndex {
                mainnet_flag: false,
                rollup_index: 1,
                leaf_index: 733,
            },
        );
    }
}
