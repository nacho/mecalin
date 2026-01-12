use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonStep {
    pub id: u32,
    pub text: String,
    pub description: Option<String>,
    #[serde(default = "default_repetitions")]
    pub repetitions: u32,
    #[serde(default)]
    pub introduction: bool,
}

fn default_repetitions() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, glib::Boxed)]
#[boxed_type(name = "Lesson")]
pub struct Lesson {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub steps: Vec<LessonStep>,
    #[serde(default)]
    pub introduction: bool,
}

#[derive(Serialize, Deserialize)]
struct LessonsData {
    lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "Course")]
pub struct Course {
    lessons: Vec<Lesson>,
}

impl Course {
    pub fn new_with_language(language: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lessons_json = match language {
            "es" => include_str!("../data/lessons/es.json"),
            _ => include_str!("../data/lessons/us.json"),
        };
        let lessons_data: LessonsData = serde_json::from_str(lessons_json)?;
        Ok(Self {
            lessons: lessons_data.lessons,
        })
    }

    pub fn get_lessons(&self) -> &Vec<Lesson> {
        &self.lessons
    }

    pub fn get_lesson(&self, id: u32) -> Option<&Lesson> {
        self.lessons.iter().find(|lesson| lesson.id == id)
    }

    pub fn get_next_lesson(&self, current_id: u32) -> Option<&Lesson> {
        self.lessons
            .iter()
            .find(|lesson| lesson.id == current_id + 1)
    }
}

impl Default for Course {
    fn default() -> Self {
        let language = crate::utils::language_from_locale();
        Self::new_with_language(language).unwrap_or_else(|_| Self { lessons: vec![] })
    }
}
