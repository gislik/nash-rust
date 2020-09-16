use super::types::ListAccountTradesRequest;
use crate::graphql;
use crate::graphql::list_account_trades;

use graphql_client::GraphQLQuery;

impl ListAccountTradesRequest {
    pub fn make_query(&self) -> graphql_client::QueryBody<list_account_trades::Variables> {
        let get_order = list_account_trades::Variables {
            payload: list_account_trades::ListAccountTradesParams {
                before: self.before.clone(),
                limit: self.limit,
                market_name: Some(self.market.market_name()),
                range_start: self.range.as_ref().map(|x| format!("{:?}", x.start)),
                range_stop: self.range.as_ref().map(|x| format!("{:?}", x.stop)),
            },
        };
        graphql::ListAccountTrades::build_query(get_order)
    }
}
