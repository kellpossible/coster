//! This module holds the business logic for the `coster` application.

mod actions;
mod error;
mod expense;
mod settlement;
mod tab;
mod user;

pub use actions::*;
pub use error::*;
pub use expense::*;
pub use settlement::*;
pub use tab::*;
pub use user::*;

#[cfg(test)]
mod tests {
    use super::{Expense, Tab, User};
    use chrono::NaiveDate;
    use commodity::exchange_rate::ExchangeRate;
    use commodity::{Commodity, CommodityType};
    use std::rc::Rc;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn balance_simple() {
        let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());

        let user1 = Rc::from(User::new(1, "User 1", None));
        let user2 = Rc::from(User::new(2, "User 2", None));
        let user3 = Rc::from(User::new(3, "User 3", None));

        let expense = Expense::new(
            1,
            "Petrol",
            "Test",
            NaiveDate::from_ymd(2020, 2, 27),
            user1.id,
            vec![user2.id, user3.id],
            Commodity::from_str("300.0 AUD").unwrap(),
            None,
        );

        let tab = Tab::new(
            Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
            "Test",
            aud.id,
            vec![user1.clone(), user2.clone(), user3.clone()],
            vec![expense],
        );

        let settlements = tab.balance_transactions().unwrap();

        assert_eq!(2, settlements.len());

        let user2_settlement = settlements.iter().find(|s| s.sender == user2.id).unwrap();
        assert!(user2_settlement.receiver == user1.id);
        assert_eq!(
            Commodity::from_str("150.0 AUD").unwrap(),
            user2_settlement.amount
        );

        let user3_settlement = settlements.iter().find(|s| s.sender == user3.id).unwrap();
        assert!(user3_settlement.receiver == user1.id);
        assert_eq!(
            Commodity::from_str("150.0 AUD").unwrap(),
            user3_settlement.amount
        );
    }

    #[test]
    fn balance_complex() {
        let aud = Rc::from(CommodityType::from_currency_alpha3("AUD").unwrap());

        let user1 = Rc::from(User::new(1, "User 1", None));
        let user2 = Rc::from(User::new(2, "User 2", None));
        let user3 = Rc::from(User::new(3, "User 3", None));

        let expenses = vec![
            Expense::new(
                1,
                "Cheese",
                "Food",
                NaiveDate::from_ymd(2020, 2, 27),
                user1.id,
                vec![user1.id, user2.id, user3.id],
                Commodity::from_str("300.0 AUD").unwrap(),
                None,
            ),
            // user2 and user3 each owe 100.0 to user1.
            // user1 is owed 200.0
            Expense::new(
                2,
                "Pickles",
                "Food",
                NaiveDate::from_ymd(2020, 2, 27),
                user1.id,
                vec![user2.id, user3.id],
                Commodity::from_str("500.0 AUD").unwrap(),
                None::<ExchangeRate>,
            ),
            // user2 and user3 both owe 250.0 to user1.
            // user1 is owed 500.0
            Expense::new(
                3,
                "Buns",
                "Food",
                NaiveDate::from_ymd(2020, 2, 27),
                user2.id,
                vec![user1.id, user2.id, user3.id],
                Commodity::from_str("100.0 AUD").unwrap(),
                None::<ExchangeRate>,
            ),
            // user1 and user3 both owe 33.333 to user2
            // user2 is owed 66.666

            // Expected totals after all this:
            // user1 is owed a total of 666.666
            // user2 owes 283.333 to user1
            // user3 owes 383.333 to user1
            // together, the users spent a total of 900.00
            // after balancing:
            // user1 spent 133.333
            // user2 spent 383.333
            // user3 spent 383.333
        ];

        let tab = Tab::new(
            Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
            "Test",
            aud.id,
            vec![user1.clone(), user2.clone(), user3.clone()],
            expenses,
        );

        let settlements = tab.balance_transactions().unwrap();

        assert_eq!(2, settlements.len());

        let user2_settlement = settlements.iter().find(|s| s.sender == user2.id).unwrap();
        assert!(user2_settlement.receiver == user1.id);
        assert!(user2_settlement.amount.eq_approx(
            Commodity::from_str("283.33333333333 AUD").unwrap(),
            Commodity::default_epsilon()
        ));

        let user3_settlement = settlements.iter().find(|s| s.sender == user3.id).unwrap();
        assert!(user3_settlement.receiver == user1.id);
        assert!(user3_settlement.amount.eq_approx(
            Commodity::from_str("383.33333333333 AUD").unwrap(),
            Commodity::default_epsilon()
        ));
    }
}
