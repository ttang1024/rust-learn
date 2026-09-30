/// Offset-based page selection for list queries.
///
/// The limit is clamped to `1..=MAX_LIMIT` so a client can never ask the
/// database for an unbounded result set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    limit: u32,
    offset: u32,
}

impl PageRequest {
    pub const DEFAULT_LIMIT: u32 = 50;
    pub const MAX_LIMIT: u32 = 100;

    pub fn new(limit: u32, offset: u32) -> Self {
        Self {
            limit: limit.clamp(1, Self::MAX_LIMIT),
            offset,
        }
    }

    pub fn limit(self) -> u32 {
        self.limit
    }

    pub fn offset(self) -> u32 {
        self.offset
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(Self::DEFAULT_LIMIT, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_is_clamped() {
        assert_eq!(PageRequest::new(0, 0).limit(), 1);
        assert_eq!(PageRequest::new(10, 0).limit(), 10);
        assert_eq!(PageRequest::new(10_000, 0).limit(), PageRequest::MAX_LIMIT);
    }

    #[test]
    fn default_page() {
        let page = PageRequest::default();
        assert_eq!(page.limit(), PageRequest::DEFAULT_LIMIT);
        assert_eq!(page.offset(), 0);
    }
}
