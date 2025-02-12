use std::{borrow::Borrow, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{Answer, ImageRectangle, QuestPage, QuestPageElement, Question};

#[derive(
    Debug,
    thiserror::Error,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum PageParseError {
    #[error("<question> tag must be closed")]
    UnclosedQuestionTag,
    #[error("<question> tag cannot be empty")]
    EmptyQuestionTag,
    #[error("choice question variant lines must start with + or -")]
    BadChoiceFormat,
    #[error("choice question defines identical choices")]
    IdenticalChoices,
    #[error("choice question has multiple correct answers")]
    MultipleCorrect,
    #[error("choice question has no correct answer")]
    NoCorrectChoice,
    #[error("opened question must contain three lines: <opened>, correct answer, and </opened>")]
    BadOpenedFormat,
    #[error(
        "image question must contain 5 lines: <img /> tag, left offset, top offset, width, height"
    )]
    BadImageFormat,
    #[error("Failed to recognize question type")]
    UnknownQuestionType,
}

fn parse_question<'l>(
    source: impl IntoIterator<Item = &'l str>,
) -> Result<Question, PageParseError> {
    let mut lines = source.into_iter();
    let Some(first_line) = lines.next() else {
        return Err(PageParseError::EmptyQuestionTag);
    };
    if first_line.starts_with(['+', '-']) {
        let mut variants_map = HashMap::<String, u32>::new();
        let mut correct = None::<u32>;

        if first_line.starts_with('+') {
            correct = Some(0);
        }

        #[allow(clippy::items_after_statements)]
        fn parse_choice(s: &str) -> Option<&str> {
            let trimmed = s.strip_prefix(['+', '-'])?.trim();
            if trimmed.is_empty() {
                return None;
            }
            Some(trimmed)
        }

        variants_map.insert(
            parse_choice(first_line)
                .ok_or(PageParseError::BadChoiceFormat)?
                .to_owned(),
            0,
        );

        for (line, no) in lines.zip(1u32..) {
            if !line.starts_with(['+', '-']) {
                return Err(PageParseError::BadChoiceFormat);
            }

            if line.starts_with('+') {
                if correct.is_some() {
                    return Err(PageParseError::MultipleCorrect);
                }
                correct = Some(no);
            }

            let variant = parse_choice(line).ok_or(PageParseError::NoCorrectChoice)?;
            if variants_map.contains_key(variant) {
                return Err(PageParseError::IdenticalChoices);
            }
            variants_map.insert(variant.to_owned(), no);
        }

        let correct = correct.ok_or(PageParseError::NoCorrectChoice)?;

        let mut variants = vec![String::new(); variants_map.len()].into_boxed_slice();
        for (variant, no) in variants_map {
            variants[no as usize] = variant;
        }
        return Ok(Question::Choice { variants, correct });
    }

    if first_line.trim() == "<opened>" {
        let Some(correct) = lines.next() else {
            return Err(PageParseError::BadOpenedFormat);
        };

        let Some(opened_end) = lines.next() else {
            return Err(PageParseError::BadOpenedFormat);
        };
        if opened_end.trim() != "</opened>" {
            return Err(PageParseError::BadOpenedFormat);
        }

        if lines.next().is_some() {
            return Err(PageParseError::BadOpenedFormat);
        }

        return Ok(Question::Opened(correct.trim().to_owned()));
    }

    if let Some(url) = first_line
        .trim()
        .strip_prefix("<img")
        .and_then(|s| s.trim_start().strip_prefix("src=\""))
        .and_then(|s| s.strip_suffix("/>"))
        .and_then(|s| s.trim_end().strip_suffix("\""))
    {
        fn parse_starting_u32(mut s: &str) -> Option<u32> {
            if let Some((stripped_end, _)) = s.split_once(' ') {
                s = stripped_end;
            }
            s.parse::<u32>().ok()
        }
        let Some(left) = lines.next().and_then(parse_starting_u32) else {
            return Err(PageParseError::BadImageFormat);
        };
        let Some(top) = lines.next().and_then(parse_starting_u32) else {
            return Err(PageParseError::BadImageFormat);
        };
        let Some(width) = lines.next().and_then(parse_starting_u32) else {
            return Err(PageParseError::BadImageFormat);
        };
        let Some(height) = lines.next().and_then(parse_starting_u32) else {
            return Err(PageParseError::BadImageFormat);
        };

        return Ok(Question::Image {
            src: url.to_owned(),
            correct_bounds: ImageRectangle {
                left,
                top,
                width,
                height,
            },
        });
    }

    Err(PageParseError::UnknownQuestionType)
}

pub fn parse_quest_page(source: impl Borrow<str>) -> Result<QuestPage, PageParseError> {
    let mut res = Vec::<QuestPageElement>::new();
    let mut question_lines = Vec::<&str>::new();
    let mut text = String::new();

    let mut lines = <_ as Borrow<str>>::borrow(&source).lines();
    while let Some(line) = lines.next() {
        if line.trim() == "<question>" {
            if !text.is_empty() {
                res.push(QuestPageElement::Text(text.clone().into_boxed_str()));
                text.clear();
            }
            question_lines.clear();
            loop {
                let Some(line) = lines.next() else {
                    return Err(PageParseError::UnclosedQuestionTag);
                };
                if line.trim() == "</question>" {
                    let question = parse_question(question_lines.drain(..))?;
                    res.push(QuestPageElement::Question(question));
                    break;
                }
            }
        } else {
            if !text.is_empty() {
                text.push('\n');
            }

            text.push_str(line);
        }
    }
    if !text.is_empty() {
        res.push(QuestPageElement::Text(text.into_boxed_str()));
    }

    Ok(res.into_boxed_slice())
}

#[cfg(test)]
mod parse_tests {
    use super::parse_question;
    use crate::Question;

    #[test]
    fn question_parse() {
        assert_eq!(
            parse_question([]),
            Err(crate::PageParseError::EmptyQuestionTag)
        );

        assert_eq!(
            parse_question(["me-ee?"]),
            Err(crate::PageParseError::UnknownQuestionType)
        );

        assert_eq!(
            parse_question(["-"]),
            Err(crate::PageParseError::BadChoiceFormat)
        );

        assert_eq!(
            parse_question(["- a", "- b"]),
            Err(crate::PageParseError::NoCorrectChoice)
        );

        assert_eq!(
            parse_question(["+ a", "+ b"]),
            Err(crate::PageParseError::MultipleCorrect)
        );

        assert_eq!(
            parse_question(["+   a", "- b  "]),
            Ok(Question::Choice {
                variants: vec!["a".to_string(), "b".to_string()].into_boxed_slice(),
                correct: 0,
            })
        );

        assert_eq!(
            parse_question(["-a  ", "+  b  "]),
            Ok(Question::Choice {
                variants: vec!["a".to_string(), "b".to_string()].into_boxed_slice(),
                correct: 1,
            })
        );

        assert_eq!(
            parse_question([" <opened>   ", " (correct answer) ", " </opened>    "]),
            Ok(Question::Opened("(correct answer)".to_string()))
        );

        assert_eq!(
            parse_question([
                "<img src=\"some sort of url\" />",
                "32 -- comment?",
                "23 -- another comment",
                "7",
                "5 ,"
            ]),
            Ok(Question::Image {
                src: "some sort of url".to_string(),
                correct_bounds: crate::ImageRectangle {
                    left: 32,
                    top: 23,
                    width: 7,
                    height: 5
                }
            })
        );
    }
}

#[derive(
    Debug,
    thiserror::Error,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[error("Wrong answer type provided")]
pub struct WrongQuestionType;

pub fn check_answer(
    quetion: impl Borrow<Question>,
    answer: impl Borrow<Answer>,
) -> Result<bool, WrongQuestionType> {
    match (quetion.borrow(), answer.borrow()) {
        (Question::Opened(correct), Answer::Opened(answered)) if correct == answered => Ok(true),
        (
            Question::Choice {
                variants: _,
                correct,
            },
            Answer::Choice(answered),
        ) if correct == answered => Ok(true),
        (
            Question::MultipleChoice {
                variants: _,
                correct,
            },
            Answer::MultipleChoice(answered),
        ) if correct == answered => Ok(true),
        (
            Question::Image {
                src: _,
                correct_bounds,
            },
            &Answer::Image { left, top },
        ) if correct_bounds.contains(left, top) => todo!(),
        (Question::Opened(..), Answer::Opened(..))
        | (Question::Choice { .. }, Answer::Choice(..))
        | (Question::MultipleChoice { .. }, Answer::MultipleChoice(..))
        | (Question::Image { .. }, Answer::Image { .. }) => Ok(false),
        _ => Err(WrongQuestionType),
    }
}
