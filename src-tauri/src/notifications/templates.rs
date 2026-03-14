use crate::db::models::{CrawlItem, DayStats};
use rand::seq::SliceRandom;

pub fn generate_teaser(stats: &DayStats, latest: Option<&CrawlItem>) -> String {
    let mut rng = rand::thread_rng();

    if stats.total_items == 0 {
        let idle_messages = [
            "Still warming up the doomscroll engines... You keep working.",
            "Haven't found anything distracting yet. The internet is quiet. Suspicious.",
            "Zero memes so far. Is the internet broken? Either way, keep focusing.",
            "The crawler is stretching its legs. Give it a moment.",
            "Nothing to report yet. The internet is playing hard to get.",
            "No content found. Either the servers are down or everyone's being productive. Unlikely.",
            "Still loading the first batch. In the meantime: work.",
            "Scanning the internet for garbage... please stand by.",
        ];
        return idle_messages.choose(&mut rng).unwrap().to_string();
    }

    let category_teasers = match latest.map(|i| i.category.as_str()) {
        Some("meme") => vec![
            format!("Just saw the funniest meme. But you've got work to do."),
            format!("Found a meme that made me snort. You can see it later."),
            format!("The memes are fire today. You'll love them. Later though."),
            format!("I've been looking at memes so you don't have to. You're welcome."),
            format!("Just found a meme that would ruin your focus. Saving it for later."),
            format!("The meme pipeline is flowing. You'll get yours at 5pm."),
            format!("Saw something that would make you exhale sharply through your nose. Later."),
            format!("A truly mid meme just came through. But in the current meta, that's fine."),
            format!("Found a meme so good I almost told you about it. Almost."),
            format!("Your meme reserves are growing nicely. Stay on task."),
            format!("Another meme acquired. The collection grows. You work."),
            format!("Intercepted a meme before it could distract you. I'm your concentration firewall."),
        ],
        Some("joke") => vec![
            format!(
                "Read {} dad jokes. None were funny. You're not missing anything.",
                stats.jokes_found
            ),
            format!(
                "Why did the programmer quit? Because he didn't get arrays. Anyway, keep coding."
            ),
            format!("I just read a joke so bad it looped back around to being good. Focus."),
            format!("Found a dad joke. It's terrible. You'll love it later."),
            format!(
                "{} jokes in and my sense of humor is legally dead. You're not missing much.",
                stats.jokes_found
            ),
            format!("Found a joke so bad it might actually be a war crime. Saving it."),
            format!("Another punchline landed. It hurt. Keep working."),
            format!("The joke quality today is... aspirational. Stay focused."),
            format!("Read a joke that made me question whether humor was a mistake. You'll see it later."),
            format!("Just suffered through a pun. The damage is done. Keep coding."),
            format!(
                "If bad jokes were currency, I'd be rich. {} and counting.",
                stats.jokes_found
            ),
            format!("Someone on the internet tried to be funny. Bless their heart. Anyway — work."),
        ],
        Some("news") => vec![
            format!(
                "Checked the news {} times. Nothing happened. Keep working.",
                stats.news_checked
            ),
            format!("Breaking: absolutely nothing important happened. Stay focused."),
            format!("The news is still depressing. I'm reading it so you don't have to."),
            format!("World still turning. No alien invasion yet. Back to work."),
            format!("Scanned the headlines. Everything is fine. Probably. Keep working."),
            format!("The news cycle continues its eternal rotation. Nothing for you to worry about."),
            format!(
                "{} articles checked. The world is still out there. Unfortunately.",
                stats.news_checked
            ),
            format!("News update: same stuff, different day. Stay productive."),
            format!("Breaking news: nothing is broken. Well, nothing new. Focus."),
            format!("Monitoring the news so you can maintain plausible deniability. Keep at it."),
            format!("I've read the news. You don't need to. Trust me on this one."),
            format!("Another news cycle absorbed. Your ignorance is bliss and also intentional."),
        ],
        Some("video") => vec![
            format!("Just watched a cat video. It was adorable. You can't see it yet."),
            format!(
                "Found a video that's {} seconds of pure joy. Save it for later.",
                rand::random::<u32>() % 40 + 10
            ),
            format!("The internet has blessed us with another funny video. Keep grinding."),
            format!("Auto-played a video so you didn't have to. It was {} seconds long. You'll survive.", rand::random::<u32>() % 40 + 10),
            format!("Found a video. Is it important? No. Is it entertaining? Debatable. Keep working."),
            format!("Another video consumed on your behalf. The algorithm thanks you for your sacrifice."),
            format!("Watched a video that was either art or a waste of time. The line is thin."),
            format!("A video just tried to grab your attention. I intercepted it. You're safe."),
            format!("Previewed a {}-second video. Your future self will decide if it was worth it.", rand::random::<u32>() % 40 + 10),
        ],
        Some("gossip") => vec![
            format!("Celebrity did a thing. It's juicy. But you have deadlines."),
            format!("Entertainment news update: drama happened. You can read about it at 5pm."),
            format!("Someone famous did something dumb. Nothing new. Keep working."),
            format!("A celebrity made a choice. The internet has opinions. You have a deadline."),
            format!("Drama in the entertainment world. I'll brief you after hours."),
            format!("Someone's publicist is having a bad day. You can read about it later."),
            format!("The gossip mill turns. Your productivity doesn't have to stop for it."),
            format!("Celebrity news intercepted. Relevance to your life: zero. Filed anyway."),
            format!("Someone famous did something on social media. The world reacted. You didn't. Good."),
        ],
        _ => vec![format!(
            "Found {} interesting things while you've been working. Stay focused!",
            stats.total_items
        )],
    };

    let general_teasers = [
        format!(
            "I've found {} things today. {} memes, {} jokes. You can binge later.",
            stats.total_items, stats.memes_found, stats.jokes_found
        ),
        format!(
            "Doomscrolled for you: {} items and counting. Estimated {} minutes saved.",
            stats.total_items, stats.estimated_time_saved_minutes as u32
        ),
        format!(
            "Your procrastination proxy is hard at work. {} items catalogued.",
            stats.total_items
        ),
        format!(
            "I've checked {} news articles, {} memes, and {} videos. You've checked zero. Perfect.",
            stats.news_checked, stats.memes_found, stats.videos_found
        ),
        format!(
            "{} items collected and counting. Your curated doomscroll awaits.",
            stats.total_items
        ),
        format!(
            "The Cazzmachine has catalogued {} distractions. They're not going anywhere.",
            stats.total_items
        ),
        format!(
            "{} things found so far. That's roughly {} minutes of internet I absorbed for you.",
            stats.total_items, stats.estimated_time_saved_minutes as u32
        ),
        format!(
            "Status report: {} memes, {} jokes, {} news articles. All handled. You're clear.",
            stats.memes_found, stats.jokes_found, stats.news_checked
        ),
        format!(
            "{} items deep. The internet has no bottom and neither does my patience.",
            stats.total_items
        ),
        format!(
            "Your doomscroll backlog grows: {} items. You keep doing actual work.",
            stats.total_items
        ),
        format!(
            "I've been busy: {} items consumed, {} minutes of distraction neutralized.",
            stats.total_items, stats.estimated_time_saved_minutes as u32
        ),
        format!(
            "Running tally: {} items. At this rate, you might actually get things done today.",
            stats.total_items
        ),
        format!(
            "{} items of slop processed. Human-made or AI-generated? Not that it matters.",
            stats.total_items
        ),
    ];

    let use_category = rand::random::<f32>() > 0.4;
    if use_category && !category_teasers.is_empty() {
        category_teasers.choose(&mut rng).unwrap().clone()
    } else {
        general_teasers.choose(&mut rng).unwrap().clone()
    }
}
