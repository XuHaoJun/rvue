//! Hacker News example with create_resource

use rudo_gc::{Trace, Visitor};
use rvue::async_runtime::create_resource;
use rvue::event::types::PointerButtonEvent;
use rvue::impl_gc_capture;
use std::io::Write;

use rvue::prelude::*;
use rvue_macro::view;
use rvue_style::{BorderWidth, Height, ReactiveStyles, Size, Width};
use serde::Deserialize;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct Story {
    id: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    score: i32,
    #[serde(default)]
    by: String,
    #[serde(default)]
    time: i64,
}

unsafe impl Trace for Story {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

impl_gc_capture!(Story);

fn get_mock_stories() -> Vec<Story> {
    vec![
        Story {
            id: 47012717,
            title: "Zig – io_uring and Grand Central Dispatch".to_string(),
            url: "".to_string(),
            score: 100,
            by: "testuser".to_string(),
            time: 1700000000,
        },
        Story {
            id: 46954696,
            title: "Show HN: I spent 3 years reverse-engineering a 40 yo stock market sim"
                .to_string(),
            url: "".to_string(),
            score: 200,
            by: "hnuser".to_string(),
            time: 1700000001,
        },
        Story {
            id: 47011567,
            title: "Show HN: SQL-tap – Real-time SQL traffic viewer".to_string(),
            url: "".to_string(),
            score: 150,
            by: "dev".to_string(),
            time: 1700000002,
        },
        Story {
            id: 46965829,
            title: "The Three Year Myth".to_string(),
            url: "".to_string(),
            score: 80,
            by: "reader".to_string(),
            time: 1700000003,
        },
        Story {
            id: 46936671,
            title: "Understanding the Go Compiler: The Linker".to_string(),
            url: "".to_string(),
            score: 120,
            by: "gopher".to_string(),
            time: 1700000004,
        },
        Story {
            id: 47013059,
            title: "Ars Technica makes up quotes from Matplotlib maintainer".to_string(),
            url: "".to_string(),
            score: 300,
            by: "news".to_string(),
            time: 1700000005,
        },
        Story {
            id: 47000505,
            title: "Babylon 5 is now free to watch on YouTube".to_string(),
            url: "".to_string(),
            score: 500,
            by: "tvfan".to_string(),
            time: 1700000006,
        },
        Story {
            id: 47012964,
            title: "YouTube as Storage".to_string(),
            url: "".to_string(),
            score: 90,
            by: "hacker".to_string(),
            time: 1700000007,
        },
        Story {
            id: 46963047,
            title: "The mathematics of compression in database systems".to_string(),
            url: "".to_string(),
            score: 70,
            by: "math".to_string(),
            time: 1700000008,
        },
        Story {
            id: 47008163,
            title: "Show HN: Data Engineering Book".to_string(),
            url: "".to_string(),
            score: 110,
            by: "book".to_string(),
            time: 1700000009,
        },
    ]
}

async fn fetch_top_stories() -> Result<Vec<Story>, String> {
    // Use mock HTTP if MOCK_HTTP env var is set
    if std::env::var("MOCK_HTTP").is_ok() {
        println!("[MOCK] Using mock HTTP client");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("[MOCK] Sleep done, creating stories");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let stories = get_mock_stories();
        println!("[MOCK] Stories created, count={}", stories.len());
        std::io::Write::flush(&mut std::io::stdout()).ok();
        for (i, story) in stories.iter().enumerate() {
            println!("[MOCK] Fetched story[{}]: id={}, title={}", i, story.id, story.title);
        }
        println!("[MOCK] Total stories fetched: {}", stories.len());
        std::io::Write::flush(&mut std::io::stdout()).ok();
        return Ok(stories);
    }

    // Real HTTP client
    let client = reqwest::Client::new();
    let response = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = response.json().await.map_err(|e| e.to_string())?;

    let story_ids: Vec<i64> = ids.into_iter().take(10).collect();

    let mut stories = Vec::new();
    for (i, id) in story_ids.iter().enumerate() {
        let story_response = client
            .get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if let Ok(story) = story_response.json::<Story>().await {
            println!("Fetched story[{}]: id={}, title={}", i, story.id, story.title);
            stories.push(story);
        } else {
            println!("Failed to fetch story[{}]: id={}", i, id);
        }
    }

    println!("Total stories fetched: {}", stories.len());
    Ok(stories)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single EnvFilter so LevelFilter::current() is set (needed for tracing-log).
    // Fallback to "debug" when RUST_LOG is unset so log::debug! from rvue is visible.
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(""))
        .add_directive("rvue=debug".parse().unwrap())
        .add_directive("rudo_gc=debug".parse().unwrap());

    tracing_subscriber::registry().with(fmt::layer()).with(env_filter).init();

    // Bridge log crate to tracing so we can see log::debug! output from rvue
    tracing_log::LogTracer::init().ok();

    let app_view = hacker_news_app();
    rvue::run_app_with_stylesheet(|| app_view, None)?;
    Ok(())
}

fn hacker_news_app() -> ViewStruct {
    let (page, _) = create_signal(0usize);

    let resource = create_resource(page, |_page: usize| async move { fetch_top_stories().await });

    let resource_for_refresh = resource.clone();
    let refresh = move || {
        resource_for_refresh.refetch();
    };

    let stories = create_memo(move || {
        let state = resource.get();
        state.data().cloned().unwrap_or_default()
    });

    view! {
        <Flex
            gap=12.0
            styles=ReactiveStyles::new()
                .set_flex_direction(FlexDirection::Column)
                .set_gap(Gap(12.0))
                .set_padding(Padding(16.0))
                .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                .set_overflow_y(Overflow::Auto)
                .set_overflow_x(Overflow::Auto)
                .set_width(Width(Size::Percent(100.0)))
                .set_height(Height(Size::Percent(100.0)))
        >
            <Flex
                styles=ReactiveStyles::new()
                    .set_gap(Gap(8.0))
                    .set_align_items(AlignItems::Center)
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
            >
                <Text content="Hacker News" font_size=24.0 font_weight="bold" />
            </Flex>
            <Flex
                styles=ReactiveStyles::new()
                    .set_gap(Gap(8.0))
                    .set_align_items(AlignItems::Center)
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
            >
                <Text content="Top 10 Stories" font_size=12.0 color="#666" />
            </Flex>
            <Flex
                styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(1.0)))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
            />
            <For each=stories key=|s: &Story| s.id view={|s| view! {
                <Flex
                    styles=ReactiveStyles::new()
                        .set_flex_direction(FlexDirection::Row)
                        .set_gap(Gap(12.0))
                        .set_padding(Padding(16.0))
                        .set_border_radius(BorderRadius(8.0))
                        .set_border_width(BorderWidth(1.0))
                        .set_border_style(BorderStyle::Solid)
                        .set_border_color(BorderColor(Color::rgb(200, 200, 200)))
                        .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                        .set_width(Width(Size::Percent(100.0)))
                        .set_overflow_x(Overflow::Auto)
                >
                     <Text content=s.title.clone() font_size=15.0 font_weight="500" />
                     <Text content=format!("Score: {}", s.score) font_size=12.0 color="#666" />
                     <Text content=format!("by {}", s.by) font_size=12.0 color="#666" />
                </Flex>
            }}/>
            <Flex
                styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(1.0)))
                    .set_background_color(BackgroundColor(Color::rgb(224, 224, 224)))
            />
            <Flex
                styles=ReactiveStyles::new()
                    .set_gap(Gap(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
            >
                <Button on_click=move |_: &PointerButtonEvent| {
                    refresh();
                }>
                    <Text content="Refresh" font_size=14.0 />
                </Button>
            </Flex>
        </Flex>
    }
}
