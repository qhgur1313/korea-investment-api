use reqwest::Response;
use url::Url;

use crate::types::request::stock::quote::{
    DailyPriceParameter, PeriodicPriceParameter, VolumeRankParameter,
};
use crate::types::response::stock::quote::{
    DailyPriceResponse, PeriodicPriceResponse, VolumeRankResponse,
};
use crate::types::{Account, Environment, MarketCode, PeriodCode, TrId};
use crate::{auth, Error};

#[derive(Clone)]
pub struct Quote {
    client: reqwest::Client,
    endpoint_url: String,
    environment: Environment,
    auth: auth::Auth,
    account: Account,
}

impl Quote {
    /// 국내주식시세에 관한 API
    /// [국내주식시세](https://apiportal.koreainvestment.com/apiservice/apiservice-domestic-stock-quotations#L_07802512-4f49-4486-91b4-1050b6f5dc9d)
    pub fn new(
        client: &reqwest::Client,
        environment: Environment,
        auth: auth::Auth,
        account: Account,
    ) -> Result<Self, Error> {
        let endpoint_url = match environment {
            Environment::Real => "https://openapi.koreainvestment.com:9443",
            Environment::Virtual => "https://openapivts.koreainvestment.com:29443",
        }
        .to_string();
        Ok(Self {
            client: client.clone(),
            endpoint_url,
            environment,
            auth,
            account,
        })
    }

    /// 주식현재가 일자별[v1_국내주식-010]
    pub async fn daily_price(
        &self,
        market_code: MarketCode,
        shortcode: &str,
        period_code: PeriodCode,
        is_adjust_price: bool,
    ) -> Result<DailyPriceResponse, Error> {
        let tr_id = TrId::DailyPrice;
        let param = DailyPriceParameter::new(
            market_code,
            shortcode.to_string(),
            period_code,
            is_adjust_price,
        );
        let url = format!(
            "{}/uapi/domestic-stock/v1/quotations/inquire-daily-price",
            self.endpoint_url
        );
        let url = reqwest::Url::parse_with_params(&url, &param.into_iter())?;
        Ok(self.send_request(url, tr_id).await?.json().await?)
    }

    pub async fn periodic_price(
        &self,
        market_code: MarketCode,
        shortcode: &str,
        period_code: PeriodCode,
        start_day: &str, // YYYYMMDD
        end_day: &str,   // YYYYMMDD
        is_adjust_price: bool,
    ) -> Result<PeriodicPriceResponse, Error> {
        let tr_id = TrId::PeriodicPrice;
        let url = format!(
            "{}/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice",
            self.endpoint_url
        );
        let param = PeriodicPriceParameter::new(
            market_code,
            shortcode.to_string(),
            start_day.to_string(),
            end_day.to_string(),
            period_code,
            is_adjust_price,
        );
        let url = reqwest::Url::parse_with_params(&url, &param.into_iter())?;
        Ok(self.send_request(url, tr_id).await?.json().await?)
    }

    /// 거래량순위[v1_국내주식-047]
    pub async fn volume_rank(
        &self,
        params: VolumeRankParameter,
    ) -> Result<VolumeRankResponse, Error> {
        let tr_id = TrId::VolumeRank;
        let url = format!(
            "{}/uapi/domestic-stock/v1/quotations/volume-rank",
            "https://openapi.koreainvestment.com:9443", // no VirtualMarket support
        );
        let url = reqwest::Url::parse_with_params(&url, &params.into_iter())?;
        Ok(self.send_request(url, tr_id).await?.json().await?)
    }

    async fn send_request(&self, url: Url, tr_id: TrId) -> Result<Response, Error> {
        Ok(self
            .client
            .get(url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                match self.auth.get_token() {
                    Some(token) => format!("Bearer {}", token),
                    None => {
                        return Err(Error::AuthInitFailed("token"));
                    }
                },
            )
            .header("appkey", self.auth.get_appkey())
            .header("appsecret", self.auth.get_appsecret())
            .header("tr_id", Into::<String>::into(tr_id))
            .header("custtype", "P")
            .send()
            .await?)
    }
}
