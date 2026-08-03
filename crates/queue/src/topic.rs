use crate::error::{QueueError, QueueResult};

pub fn validate_topic(value: &str) -> QueueResult<()> {
    validate_words(value, false)
}

pub fn validate_pattern(value: &str) -> QueueResult<()> {
    validate_words(value, true)
}

pub fn topic_matches(pattern: &str, topic: &str) -> bool {
    if validate_pattern(pattern).is_err() || validate_topic(topic).is_err() {
        return false;
    }
    let pattern = pattern.split('.').collect::<Vec<_>>();
    let topic = topic.split('.').collect::<Vec<_>>();
    match_words(&pattern, &topic)
}

fn validate_words(value: &str, pattern: bool) -> QueueResult<()> {
    if value.is_empty() {
        return Err(QueueError::InvalidTopic("Queue topic is empty".to_string()));
    }
    for word in value.split('.') {
        if word.is_empty() {
            return Err(QueueError::InvalidTopic(
                "Queue topic contains an empty word".to_string(),
            ));
        }
        if pattern && matches!(word, "*" | "#") {
            continue;
        }
        if !word
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        {
            return Err(QueueError::InvalidTopic(
                "Queue topic contains unsupported characters".to_string(),
            ));
        }
    }
    Ok(())
}

fn match_words(pattern: &[&str], topic: &[&str]) -> bool {
    let Some((head, tail)) = pattern.split_first() else {
        return topic.is_empty();
    };
    match *head {
        "#" => (0..=topic.len()).any(|index| match_words(tail, &topic[index..])),
        "*" => !topic.is_empty() && match_words(tail, &topic[1..]),
        value => topic.first() == Some(&value) && match_words(tail, &topic[1..]),
    }
}
