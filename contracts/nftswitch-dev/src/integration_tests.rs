#[cfg(test)]
mod tests {
    use crate::{
        msg::{ConfigResponse, ExecuteMsg, QueryMsg, TradeResponse, TradesResponse},
        ContractError,
    };
    use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, Empty, Uint128};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw721_base::{
        ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as Cw721InstantiateMsg, MintMsg,
    };
    use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

    fn custom_mock_app() -> App {
        App::default()
    }

    pub fn contract_trade() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_nft() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    const TOKEN_ID: u32 = 100;

    const INITIAL_BALANCE: u128 = 2000;

    const NATIVE_DENOM: &str = "uluna";

    fn setup_contract(router: &mut App, creator: &Addr) -> Result<(Addr, Addr), ContractError> {
        let cw_template_id = router.store_code(contract_trade());

        let msg = crate::msg::InstantiateMsg {
            admin: "admin".to_string(),
            fee_admin: "fee_admin".to_string(),
            commission_addr: "commission_addr".to_string(),
            buyer_fee: Decimal::from_ratio(15u128, 1000u128),
            seller_fee: Decimal::from_ratio(15u128, 1000u128),
            listing_fee: Uint128::from(10u32),
        };

        let trade = router
            .instantiate_contract(
                cw_template_id,
                creator.clone(),
                &msg,
                &[],
                "Cw_template",
                None,
            )
            .unwrap();

        println!("trade: {:?}", trade);

        //set up contract nft
        let cw721_id = router.store_code(contract_nft());

        let msg = Cw721InstantiateMsg {
            name: String::from("Skeleton punk"),
            symbol: String::from("SP"),
            minter: creator.to_string(),
        };

        let collection = router
            .instantiate_contract(cw721_id, creator.clone(), &msg, &[], "collection", None)
            .unwrap();

        println!("collection: {:?}", collection);

        Ok((trade, collection))
    }

    fn setup_accounts(router: &mut App) -> Result<(Addr, Addr, Addr), ContractError> {
        let admin: Addr = Addr::unchecked("admin");
        let seller: Addr = Addr::unchecked("seller");
        let buyer: Addr = Addr::unchecked("buyer");
        let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);

        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: admin.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: seller.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: buyer.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        let admin_native_balances = router.wrap().query_all_balances(admin.clone()).unwrap();
        assert_eq!(admin_native_balances, funds);
        let buyer_native_balances = router.wrap().query_all_balances(seller.clone()).unwrap();
        assert_eq!(buyer_native_balances, funds);
        let seller_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(seller_native_balances, funds);

        Ok((admin, seller, buyer))
    }

    fn mint_for(router: &mut App, admin: &Addr, seller: &Addr, collection: &Addr, token_id: u32) {
        let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
            token_id: token_id.to_string(),
            owner: seller.clone().to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Empty {},
        });
        let res = router.execute_contract(
            admin.clone(),
            collection.clone(),
            &mint_for_creator_msg,
            &[],
        );
        assert!(res.is_ok());
    }

    fn approve(
        router: &mut App,
        seller: &Addr,
        collection: &Addr,
        trade_contract: &Addr,
        token_id: u32,
    ) {
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: trade_contract.to_string(),
            token_id: token_id.to_string(),
            expires: None,
        };
        let res = router.execute_contract(seller.clone(), collection.clone(), &approve_msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn try_create_and_execute_trade() {
        let mut router = custom_mock_app();

        // Setup intial accounts
        let (admin, seller, buyer) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, collection) = setup_contract(&mut router, &admin).unwrap();

        // Mint NFT for seller
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID);
        approve(&mut router, &seller, &collection, &trade_contract, TOKEN_ID);

        // Should error with create trade wrong denom
        const WRONG_DENOM: &str = "wrong_denom";
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(1000, WRONG_DENOM),
        };
        let res =
            router.execute_contract(seller.clone(), trade_contract.clone(), &create_trade, &[]);
        assert!(res.is_err());

        // Should error with create trade wrong denom
        const WRONG_TOKEN_ID: i32 = 456;
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: WRONG_TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(1000, NATIVE_DENOM),
        };
        let res =
            router.execute_contract(seller.clone(), trade_contract.clone(), &create_trade, &[]);
        assert!(res.is_err());

        //Create trade successfully
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(1000, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        let trade_msg = QueryMsg::GetTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        let res: TradeResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &trade_msg)
            .unwrap();

        assert_eq!(res.trade.clone().unwrap().buyer, buyer);
        assert_eq!(res.trade.clone().unwrap().seller, seller);
        assert_eq!(res.trade.clone().unwrap().price, coin(1000, NATIVE_DENOM));
        assert_eq!(res.trade.clone().unwrap().nft_collection, collection);
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());

        //confirm by fee admin
        let execute_confirm_by_fee_admin = ExecuteMsg::ConfirmTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            seller_fee_pct: Decimal::from_ratio(15u128, 1000u128),
            buyer_fee_pct: Decimal::from_ratio(15u128, 1000u128),
            is_confirmed_by_fee_admin: true,
        };
        let _res = router.execute_contract(
            Addr::unchecked("fee_admin".to_string()),
            trade_contract.clone(),
            &execute_confirm_by_fee_admin,
            &[],
        );

        //buyer make a trade
        let execute_trade_msg = ExecuteMsg::ExecuteTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        // Should error with create trade wrong amount. Expected amount 1000(price) + 15(fee) = 1015
        let res = router.execute_contract(
            buyer.clone(),
            trade_contract.clone(),
            &execute_trade_msg,
            &coins(1011, NATIVE_DENOM),
        );
        assert!(res.is_err());

        //Execute trade successfully
        let res = router.execute_contract(
            buyer.clone(),
            trade_contract.clone(),
            &execute_trade_msg,
            &coins(1015, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        //check seller has been paid
        let seller_balances = router.wrap().query_all_balances(seller.clone()).unwrap();
        //seller get 985 from buyer (1000 - 15(seller_fee) - 10(buyer_fee));
        assert_eq!(seller_balances, coins(2975, NATIVE_DENOM));

        //check commission address has been paid
        let commission_addr_bal = router
            .wrap()
            .query_all_balances(Addr::unchecked("commission_addr"))
            .unwrap();
        //commission addr get 15 fee from seller and 15 fee from buyer = 30 coin
        assert_eq!(commission_addr_bal, coins(30, NATIVE_DENOM));

        //check fee admin has been paid
        let commission_addr_bal = router
            .wrap()
            .query_all_balances(Addr::unchecked("fee_admin"))
            .unwrap();
        //fee admin get 10 listing fee
        assert_eq!(commission_addr_bal, coins(10, NATIVE_DENOM));

        //check nft has been transfered to buyer
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(collection, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, buyer.to_string());
    }

    #[test]
    fn try_create_and_execute_trade_with_zero_price() {
        let mut router = custom_mock_app();

        // Setup intial accounts
        let (admin, seller, buyer) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, collection) = setup_contract(&mut router, &admin).unwrap();

        // Mint NFT for seller
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID);
        approve(&mut router, &seller, &collection, &trade_contract, TOKEN_ID);

        //Create trade with zero price
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(0, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        println!("{:?}", res);

        assert!(res.is_ok());

        let trade_msg = QueryMsg::GetTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        let res: TradeResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &trade_msg)
            .unwrap();

        assert_eq!(res.trade.clone().unwrap().buyer, buyer);
        assert_eq!(res.trade.clone().unwrap().seller, seller);
        assert_eq!(res.trade.clone().unwrap().price, coin(0, NATIVE_DENOM));
        assert_eq!(res.trade.clone().unwrap().nft_collection, collection);
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());

        //confirm by fee admin
        let execute_confirm_by_fee_admin = ExecuteMsg::ConfirmTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            seller_fee_pct: Decimal::from_ratio(15u128, 1000u128),
            buyer_fee_pct: Decimal::from_ratio(15u128, 1000u128),
            is_confirmed_by_fee_admin: true,
        };
        let res = router.execute_contract(
            Addr::unchecked("fee_admin".to_string()),
            trade_contract.clone(),
            &execute_confirm_by_fee_admin,
            &[],
        );
        assert!(res.is_ok());

        // buyer make a trade
        let execute_trade_msg = ExecuteMsg::ExecuteTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        //Execute trade succesfully
        let res = router.execute_contract(
            buyer.clone(),
            trade_contract.clone(),
            &execute_trade_msg,
            &[],
        );
        assert!(res.is_ok());

        //check seller has been paid
        let seller_balances = router.wrap().query_all_balances(seller.clone()).unwrap();

        //seller get 1990 from buyer (2000 - 10(listing fee));

        assert_eq!(seller_balances, coins(1990, NATIVE_DENOM));

        //check commission address has been paid
        let commission_addr_bal = router
            .wrap()
            .query_all_balances(Addr::unchecked("commission_addr"))
            .unwrap();
        //commission addr get nothing
        assert_eq!(commission_addr_bal, []);

        //check fee admin address has been paid
        let fee_admin_bal = router
            .wrap()
            .query_all_balances(Addr::unchecked("fee_admin"))
            .unwrap();
        //commission addr get nothing
        assert_eq!(fee_admin_bal, coins(10, NATIVE_DENOM));

        //check nft has been transfered to buyer
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(collection, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, buyer.to_string());
    }

    #[test]
    fn try_cancel_trade() {
        let mut router = custom_mock_app();

        // Setup intial accounts
        let (admin, seller, buyer) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, collection) = setup_contract(&mut router, &admin).unwrap();

        // Mint NFT for seller
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID);
        approve(&mut router, &seller, &collection, &trade_contract, TOKEN_ID);

        //Create trade successfully
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(1000, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        let trade_msg = QueryMsg::GetTrade {
            buyer: buyer.to_string(),
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        let res: TradeResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &trade_msg)
            .unwrap();

        assert_eq!(res.trade.clone().unwrap().buyer, buyer);
        assert_eq!(res.trade.clone().unwrap().seller, seller);
        assert_eq!(res.trade.clone().unwrap().price, coin(1000, NATIVE_DENOM));
        assert_eq!(res.trade.clone().unwrap().nft_collection, collection);
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());
        assert_eq!(res.trade.clone().unwrap().nft_id, TOKEN_ID.to_string());

        //cancel trade
        let execute_cancel_trade = ExecuteMsg::CancelTrade {
            buyer: Some(buyer.to_string()),
            seller: None,
            nft_collection: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
        };

        //Execute cancel succesfully
        let res = router.execute_contract(
            buyer.clone(),
            trade_contract.clone(),
            &execute_cancel_trade,
            &[],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn try_update_config() {
        let mut router = custom_mock_app();

        // Setup intial accounts
        let (admin, _, _) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, _) = setup_contract(&mut router, &admin).unwrap();

        let config_msg = QueryMsg::GetConfig {};
        let res: ConfigResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &config_msg)
            .unwrap();

        assert_eq!(res.admin, admin);
        assert_eq!(res.fee_admin, Addr::unchecked("fee_admin"));
        assert_eq!(res.commission_addr, Addr::unchecked("commission_addr"));
        assert_eq!(res.e_break, false);
        assert_eq!(res.buyer_fee, Decimal::from_ratio(15u128, 1000u128));
        assert_eq!(res.seller_fee, Decimal::from_ratio(15u128, 1000u128));

        //update admin
        let update_config_msg = ExecuteMsg::UpdateConfig {
            admin: Some("new_admin".to_string()),
            fee_admin: None,
            commission_addr: None,
            buyer_fee: None,
            seller_fee: None,
            listing_fee: None,
            e_break: None,
        };
        let res = router.execute_contract(
            admin.clone(),
            trade_contract.clone(),
            &update_config_msg,
            &[],
        );
        assert!(res.is_ok());
        let res: ConfigResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &config_msg)
            .unwrap();
        assert_eq!(res.admin, Addr::unchecked("new_admin"));

        //update other option
        let update_config_msg = ExecuteMsg::UpdateConfig {
            admin: None,
            fee_admin: Some("new_fee_admin".to_string()),
            commission_addr: Some("new_commission_addr".to_string()),
            buyer_fee: Some(Decimal::from_ratio(2u128, 100u128)),
            seller_fee: Some(Decimal::from_ratio(2u128, 100u128)),
            listing_fee: Some(Uint128::from(100000u32)),
            e_break: Some(true),
        };

        let res = router.execute_contract(
            Addr::unchecked("new_admin".to_string()),
            trade_contract.clone(),
            &update_config_msg,
            &[],
        );
        assert!(res.is_ok());
        let res: ConfigResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.clone(), &config_msg)
            .unwrap();
        assert_eq!(res.admin, Addr::unchecked("new_admin"));
        assert_eq!(res.fee_admin, Addr::unchecked("new_fee_admin"));
        assert_eq!(res.commission_addr, Addr::unchecked("new_commission_addr"));
        assert_eq!(res.e_break, true);
        assert_eq!(res.buyer_fee, Decimal::from_ratio(2u128, 100u128));
        assert_eq!(res.seller_fee, Decimal::from_ratio(2u128, 100u128));
        assert_eq!(res.listing_fee, Uint128::from(100000u32));
    }

    #[test]
    fn try_query_trades_by_seller() {
        let mut router = custom_mock_app();

        // Setup initial accounts
        let (admin, seller, buyer) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, collection) = setup_contract(&mut router, &admin).unwrap();

        // Mint NFT for creator
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID);
        approve(&mut router, &seller, &collection, &trade_contract, TOKEN_ID);
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID + 1);
        approve(
            &mut router,
            &seller,
            &collection,
            &trade_contract,
            TOKEN_ID + 1,
        );
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID + 2);
        approve(
            &mut router,
            &seller,
            &collection,
            &trade_contract,
            TOKEN_ID + 2,
        );

        // Seller lists their token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(100, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        // Seller lists their token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: (TOKEN_ID + 1).to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(110, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        // Seller lists another token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: (TOKEN_ID + 2).to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(120, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());
        // seller 2 should have 3 list trade
        let query_trade_by_seller_msg = QueryMsg::GetTradesBySeller {
            seller: seller.clone(),
            limit: None,
        };
        let res: TradesResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.to_string(), &query_trade_by_seller_msg)
            .unwrap();
        assert_eq!(res.trades.len(), 3usize);
    }

    #[test]
    fn try_query_trades_by_buyer() {
        let mut router = custom_mock_app();

        // Setup initial accounts
        let (admin, seller, buyer) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (trade_contract, collection) = setup_contract(&mut router, &admin).unwrap();

        //create seller 2
        let seller_2 = Addr::unchecked("seller_2");

        //init balance for seller 2
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: seller_2.to_string(),
                    amount: vec![coin(100u128, NATIVE_DENOM)],
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        // Mint NFT for creator
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID);
        approve(&mut router, &seller, &collection, &trade_contract, TOKEN_ID);
        mint_for(&mut router, &admin, &seller, &collection, TOKEN_ID + 1);
        approve(
            &mut router,
            &seller,
            &collection,
            &trade_contract,
            TOKEN_ID + 1,
        );
        mint_for(&mut router, &admin, &seller_2, &collection, TOKEN_ID + 2);
        approve(
            &mut router,
            &seller_2,
            &collection,
            &trade_contract,
            TOKEN_ID + 2,
        );

        // Seller lists their token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: TOKEN_ID.to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(100, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        // Seller lists their token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: (TOKEN_ID + 1).to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(110, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );
        assert!(res.is_ok());

        // Seller lists another token for sale
        let create_trade = ExecuteMsg::CreateTrade {
            nft_addr: collection.to_string(),
            nft_id: (TOKEN_ID + 2).to_string(),
            buyer_addr: buyer.to_string(),
            sale_price: coin(120, NATIVE_DENOM),
        };
        let res = router.execute_contract(
            seller_2.clone(),
            trade_contract.clone(),
            &create_trade,
            &coins(10, NATIVE_DENOM),
        );

        assert!(res.is_ok());
        // buyer 2 should have 3 list trade
        let query_trade_by_buyer_msg = QueryMsg::GetTradesByBuyer {
            buyer: buyer.clone(),
            limit: None,
        };
        let res: TradesResponse = router
            .wrap()
            .query_wasm_smart(trade_contract.to_string(), &query_trade_by_buyer_msg)
            .unwrap();
        assert_eq!(res.trades.len(), 3usize);
    }
}
