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
            "fr" => include_str!("../data/lessons/fr.json"),
            "gl" => include_str!("../data/lessons/gl.json"),
            "it" => include_str!("../data/lessons/it.json"),
            "pl" => include_str!("../data/lessons/pl.json"),
            "pt" => include_str!("../data/lessons/pt.json"),
            "pt_br" => include_str!("../data/lessons/pt_br.json"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_language_us() {
        let course = Course::new_with_language("us").unwrap();
        assert!(!course.get_lessons().is_empty());
    }

    #[test]
    fn test_new_with_language_es() {
        let course = Course::new_with_language("es").unwrap();
        assert!(!course.get_lessons().is_empty());
    }

    #[test]
    fn test_new_with_language_it() {
        let course = Course::new_with_language("it").unwrap();
        assert!(!course.get_lessons().is_empty());
    }

    #[test]
    fn test_new_with_language_invalid_defaults_to_us() {
        let course = Course::new_with_language("invalid").unwrap();
        assert!(!course.get_lessons().is_empty());
    }

    #[test]
    fn test_get_lesson_existing() {
        let course = Course::new_with_language("us").unwrap();
        let lesson = course.get_lesson(1);
        assert!(lesson.is_some());
        assert_eq!(lesson.unwrap().id, 1);
    }

    #[test]
    fn test_get_lesson_non_existing() {
        let course = Course::new_with_language("us").unwrap();
        let lesson = course.get_lesson(9999);
        assert!(lesson.is_none());
    }

    #[test]
    fn test_get_next_lesson() {
        let course = Course::new_with_language("us").unwrap();
        let first_lesson = course.get_lesson(1).unwrap();
        let next_lesson = course.get_next_lesson(first_lesson.id);
        assert!(next_lesson.is_some());
        assert_eq!(next_lesson.unwrap().id, 2);
    }

    #[test]
    fn test_get_next_lesson_last() {
        let course = Course::new_with_language("us").unwrap();
        let lessons = course.get_lessons();
        let last_id = lessons.last().unwrap().id;
        let next_lesson = course.get_next_lesson(last_id);
        assert!(next_lesson.is_none());
    }

    #[test]
    fn test_get_next_lesson_non_existing() {
        let course = Course::new_with_language("us").unwrap();
        let next_lesson = course.get_next_lesson(9999);
        assert!(next_lesson.is_none());
    }

    #[test]
    fn test_default_repetitions() {
        assert_eq!(default_repetitions(), 1);
    }
}
