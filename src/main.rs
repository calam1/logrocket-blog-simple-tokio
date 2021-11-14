// https://blog.logrocket.com/a-practical-guide-to-async-in-rust/
use tokio::task;
use log::*;
use std::io::Write;
use futures::future::join_all;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// simple concurrent calls - start
async fn request(n: usize) -> Result<()> {
    reqwest::get(slowly(1000)).await?;
    info!("got response {}", n);
    Ok(())
}

async fn app() -> Result<()> {
    // treat this as the main function of the async part of the program
    let resp1 = task::spawn(request(1));
    let resp2 = task::spawn(request(2));

    let _ = resp1.await??;
    let _ = resp2.await??;
    Ok(())
}

fn slowly(delay_ms: u32) -> reqwest::Url {
    let url = format!(
        "https://deelay.me//https:/{}/google.com",
        delay_ms,
    );
    reqwest::Url::parse(&url).unwrap()
}
// simple concurrent calls - end


async fn app_cpu_intensive() -> Result<()> {
    let mut futures = vec![];
    for i in 1..=10 {
        let fut = task::spawn(get_and_analyze(i));
        futures.push(fut);
    }

    let results = join_all(futures).await;

    let mut total_ones = 0;
    let mut total_zeroes = 0;

    for result in results {
        // `spawn_blocking` returns a `JoinResult` we need to unwrap first
        let ones_res: Result<(u64, u64)> = result?;
        let (ones, zeroes) = ones_res?;

        total_ones += ones;
        total_zeroes += zeroes;
    }

    info!("Ratio of ones/zeros: {:.02}",total_ones as f64 / total_zeroes as f64);
    Ok(())
}

async fn get_and_analyze(n: usize) -> Result<(u64, u64)> {
    let response: reqwest::Response = reqwest::get(slowly(1000)).await?;
    info!("Dataset {}", n);

    let txt = response.text().await?;

    // We send our analysis work to a thread where there is no runtime running
    // so we don't block the runtime by analyzing the data
    let res = task::spawn_blocking(move || analyze(&txt)).await?;
    info!("Processed {}", n);
    Ok(res)
}

// Now we want to both fetch some data and do some CPU intensive analysis on it
fn analyze(txt: &str) -> (u64, u64) {
    let txt = txt.as_bytes();

    // Let's spend as much time as we can and count them in two passes
    let ones = txt.iter().fold(0u64, |acc, b: &u8| acc + b.count_ones() as u64);
    let zeros = txt.iter().fold(0u64, |acc, b: &u8| acc + b.count_zeros() as u64);
    (ones, zeros)
}



fn main() {
    // RUST_LOG=info cargo run   


    let start = std::time::Instant::now();
    env_logger::Builder::from_default_env().format(move |buf, rec| {
        let t = start.elapsed().as_secs_f32();
        writeln!(buf, "{:.03} [{}] - {}", t, rec.level(), rec.args())
    }).init();

    let rt = tokio::runtime::Runtime::new().unwrap();

    //  simple concurrent calls start
    info!("starting simple concurrent program");
    match rt.block_on(app()) {
        Ok(_) => info!("Done"),
        Err(e) => error!("Error {}", e),
    };
    info!("finished concurrent program");
    //  simple concurrent calls end

    //  cpu intensive concurrent calls start
    info!("starting cpu intensive concurrent program");
    match rt.block_on(app_cpu_intensive()) {
        Ok(_) => info!("Done"),
        Err(e) => error!("Error {}", e),
    };
    info!("finished concurrent program");
    //  cpu intensive concurrent calls end
}
