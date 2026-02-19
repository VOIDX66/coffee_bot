mod domain;
mod infrastructure;

use infrastructure::providers::mock_provider::MockCoffeePriceProvider;
use domain::traits::coffee_price_provider::CoffeePriceProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = MockCoffeePriceProvider;

    let price = provider.get_price().await?;

    println!("Precio del café: {:?}", price);

    Ok(())
}