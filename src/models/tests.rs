#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Transaction, TransactionStatus, TransactionType, WalletAddress};
    use chrono::Utc;

    #[test]
    fn test_wallet_address_creation() {
        let address = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let wallet = WalletAddress::new(address.to_string(), Some("test_wallet".to_string()));

        assert_eq!(wallet.address, address);
        assert_eq!(wallet.label, Some("test_wallet".to_string()));
        assert!(wallet.is_active);
    }

    #[test]
    fn test_transaction_creation() {
        let signature = "5w6TpwP8pPhQ2EeFF3N7PQHQbmVjFduJR5WcKjdqSPM";
        let from_address = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let to_address = "8yKZtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";

        let transaction = Transaction::new(
            signature.to_string(),
            12345678,
            TransactionType::Native,
            from_address.to_string(),
            Some(to_address.to_string()),
            1.5,
            None,
            None,
            0.00025,
            Utc::now(),
            TransactionStatus::Confirmed,
            None,
        );

        assert_eq!(transaction.signature, signature);
        assert_eq!(transaction.from_address, from_address);
        assert_eq!(transaction.to_address, Some(to_address.to_string()));
        assert_eq!(transaction.amount, 1.5);
        assert_eq!(transaction.fee, 0.00025);
    }
}
