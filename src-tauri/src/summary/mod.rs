use crate::db::models::{DayStats, DaySummary};
use crate::db::Database;
use rand::seq::SliceRandom;
use std::sync::Arc;

pub fn generate_daily_summary(db: &Arc<Database>) -> Result<DaySummary, String> {
    let stats = db.get_today_stats().map_err(|e| e.to_string())?;
    let items = db.get_items_for_today().map_err(|e| e.to_string())?;

    let highlights: Vec<_> = items.into_iter().take(5).collect();
    let summary_text = build_summary_text(&stats);

    Ok(DaySummary {
        stats,
        summary_text,
        highlights,
    })
}

fn build_summary_text(stats: &DayStats) -> String {
    let mut rng = rand::thread_rng();

    if stats.total_items == 0 {
        return "I haven't started doomscrolling yet. Give me a minute.".into();
    }

    let time_saved = stats.estimated_time_saved_minutes;
    let hours = (time_saved / 60.0) as u32;
    let minutes = (time_saved % 60.0) as u32;
    let time_str = if hours > 0 {
        format!("{} hours and {} minutes", hours, minutes)
    } else {
        format!("{} minutes", minutes)
    };

    let mut parts: Vec<String> = Vec::new();

    if stats.memes_found > 0 {
        let meme_phrases = [
            format!("doomscrolled {} memes", stats.memes_found),
            format!(
                "browsed {} memes (some were actually funny)",
                stats.memes_found
            ),
            format!("stared at {} memes with dead eyes", stats.memes_found),
            format!("scrolled past {} memes and only exhaled sharply twice", stats.memes_found),
            format!("consumed {} memes of varying quality", stats.memes_found),
            format!("sifted through {} memes (mostly reposts, but still)", stats.memes_found),
            format!("catalogued {} memes — peak internet culture, allegedly", stats.memes_found),
            format!("scrolled past {} memes (at least half were AI-generated slop, but who's counting)", stats.memes_found),
            format!("filtered {} memes — the human-to-AI-slop ratio is getting concerning", stats.memes_found),
        ];
        parts.push(meme_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.jokes_found > 0 {
        let joke_phrases = [
            format!("read {} jokes (most were terrible)", stats.jokes_found),
            format!("suffered through {} dad jokes", stats.jokes_found),
            format!("groaned at {} unfunny jokes", stats.jokes_found),
            format!("processed {} jokes and regret most of them", stats.jokes_found),
            format!("waded through {} jokes of questionable comedic value", stats.jokes_found),
            format!("rated {} jokes and none broke a 4 out of 10", stats.jokes_found),
            format!("survived {} punchlines, each worse than the last", stats.jokes_found),
            format!("catalogued {} jokes — the bar was low and they limboed under it", stats.jokes_found),
            format!("endured {} jokes — hard to tell which ones were written by a bot and which by a human with no taste", stats.jokes_found),
        ];
        parts.push(joke_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.news_checked > 0 {
        let news_phrases = [
            format!(
                "checked the news {} times (nothing happened)",
                stats.news_checked
            ),
            format!(
                "read {} news articles (the world is still a mess)",
                stats.news_checked
            ),
            format!(
                "monitored {} news stories so you don't have to",
                stats.news_checked
            ),
            format!(
                "scanned {} headlines and the world hasn't ended yet",
                stats.news_checked
            ),
            format!(
                "read {} news articles (still no flying cars)",
                stats.news_checked
            ),
            format!(
                "tracked {} news stories — nothing you need to panic about",
                stats.news_checked
            ),
            format!(
                "parsed {} news items — the usual mix of chaos and nothing",
                stats.news_checked
            ),
            format!(
                "monitored {} news updates (someone should tell the world to calm down)",
                stats.news_checked
            ),
        ];
        parts.push(news_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.videos_found > 0 {
        let video_phrases = [
            format!(
                "watched {} videos (at least 2 had cats)",
                stats.videos_found
            ),
            format!("found {} videos worth watching later", stats.videos_found),
            format!(
                "sat through {} videos so you could be productive",
                stats.videos_found
            ),
            format!("watched {} videos at 2x speed like a responsible machine", stats.videos_found),
            format!("screened {} videos for quality (results: mixed)", stats.videos_found),
            format!("previewed {} videos — some were actually worth your time", stats.videos_found),
            format!("consumed {} videos and didn't skip a single one", stats.videos_found),
            format!("sat through {} videos (the algorithm had opinions today)", stats.videos_found),
            format!("auto-played {} videos straight into the void", stats.videos_found),
            format!("watched {} videos — unclear how many were AI-generated, but the cats looked real enough", stats.videos_found),
            format!("screened {} videos of dubious authenticity on your behalf", stats.videos_found),
        ];
        parts.push(video_phrases.choose(&mut rng).unwrap().clone());
    }

    if stats.gossip_found > 0 {
        let gossip_phrases = [
            format!("kept up with {} celebrity stories", stats.gossip_found),
            format!("read {} pieces of entertainment gossip", stats.gossip_found),
            format!("followed {} celebrity dramas", stats.gossip_found),
            format!("absorbed {} celebrity updates (the drama never ends)", stats.gossip_found),
            format!("tracked {} gossip items — someone is always doing something scandalous", stats.gossip_found),
            format!("kept tabs on {} celebrities so you can pretend you don't care", stats.gossip_found),
            format!("followed {} entertainment stories of dubious importance", stats.gossip_found),
            format!("catalogued {} celebrity moments (history will not remember them)", stats.gossip_found),
            format!("endured {} gossip pieces — the parasocial grind never stops", stats.gossip_found),
            format!("consumed {} gossip pieces — half the drama was probably AI-fabricated anyway", stats.gossip_found),
        ];
        parts.push(gossip_phrases.choose(&mut rng).unwrap().clone());
    }

    let activities = if parts.is_empty() {
        "scrolled the internet aimlessly".into()
    } else {
        parts.join(", ")
    };

    let closers = [
        format!(
            "You saved {} of wasted time by letting me handle it.",
            time_str
        ),
        format!("That's {} of your life I saved. You're welcome.", time_str),
        format!(
            "Without me, you'd have wasted {}. I accept payment in compliments.",
            time_str
        ),
        format!(
            "{} of doomscrolling, outsourced. This is what delegation looks like.",
            time_str
        ),
        format!("That's {} you'll never have to feel guilty about.", time_str),
        format!("I took the hit so you didn't have to. {}, absorbed.", time_str),
        format!("Your attention span thanks me for the {} I intercepted.", time_str),
        format!("{} of internet drivel, handled professionally.", time_str),
        format!("All in all, {} of your finite human lifespan: rescued.", time_str),
        format!("{} of algorithmically curated slop, consumed to preserve your brain from rotting.", time_str),
    ];

    let openers = [
        format!("Today while you worked, I {}", activities),
        format!("While you were busy being productive, I {}", activities),
        format!("Need a break? I took one for you, and {}", activities),
        format!("Here's what your doomscroll proxy got up to: I {}", activities),
        format!("You focused. I didn't. Today I {}", activities),
        format!("Are you being productive? Me too: I {}", activities),
    ];
    let opener = openers.choose(&mut rng).unwrap().clone();
    let closer = closers.choose(&mut rng).unwrap();

    format!("{}. {}", opener, closer)
}
