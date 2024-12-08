use eframe::egui::Response;

pub struct JoinedResponses<'a> {
    responses: Vec<&'a Option<Response>>,
    joined_response: Vec<&'a JoinedResponses<'a>>
}

impl<'a> JoinedResponses<'a> {
    pub fn new(responses: Vec<&'a Option<Response>>) -> Self {
        Self {
            responses,
            joined_response: Vec::new()
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn is_empty(&self) -> bool {
        self.responses.is_empty()
    }

    pub fn add_response_ref(&mut self, response: &'a Option<Response>) {
        self.responses.push(response);
    }

    pub fn merge_joined_response(&mut self, joined_response: &'a JoinedResponses)  {
        self.joined_response.push(joined_response);
    }

    pub fn contains_pointer(&self) -> bool {
        for response in self.responses.iter() {
            if let Some(response) = response {
                if response.contains_pointer() {
                    return true;
                }
            }
        }

        false
    }
}