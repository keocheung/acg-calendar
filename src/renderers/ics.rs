use crate::{AppResult, CalendarDocument, CalendarRenderer, RenderedCalendar};
use calcard::{
    common::{CalendarScale, PartialDateTime},
    icalendar::{
        ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarMethod, ICalendarParameter,
        ICalendarParameterName, ICalendarProperty, ICalendarValueType, Uri,
    },
};
use chrono::{Datelike, Duration, NaiveDate};

#[derive(Debug, Clone, Copy, Default)]
pub struct IcsRenderer;

impl CalendarRenderer for IcsRenderer {
    fn render(&self, calendar: &CalendarDocument) -> AppResult<RenderedCalendar> {
        let body = render_ics(calendar);
        Ok(RenderedCalendar {
            body,
            content_type: "text/calendar; charset=utf-8",
        })
    }
}

fn render_ics(calendar: &CalendarDocument) -> String {
    let mut components = Vec::with_capacity(calendar.events.len() + 1);
    let mut root = ICalendarComponent::new(ICalendarComponentType::VCalendar);
    root.add_property(ICalendarProperty::Version, "2.0");
    root.add_property(
        ICalendarProperty::Prodid,
        "-//calendar//Bangumi Game Release Calendar//EN",
    );
    root.add_property(ICalendarProperty::Calscale, CalendarScale::Gregorian);
    root.add_property(ICalendarProperty::Method, ICalendarMethod::Publish);
    root.add_property(ICalendarProperty::Name, calendar.name.as_str());
    root.add_property(
        ICalendarProperty::Other("X-WR-CALNAME".to_owned()),
        calendar.name.as_str(),
    );

    for event in &calendar.events {
        let next_day = event.date + Duration::days(1);
        let component_id = components.len() as u32 + 1;
        root.component_ids.push(component_id);

        let mut component = ICalendarComponent::new(ICalendarComponentType::VEvent);
        component.add_uid(&event.uid);
        component.add_dtstamp(PartialDateTime::now());
        component.add_property_with_params(
            ICalendarProperty::Dtstart,
            [date_value_param()],
            partial_date(event.date),
        );
        component.add_property_with_params(
            ICalendarProperty::Dtend,
            [date_value_param()],
            partial_date(next_day),
        );
        component.add_property(ICalendarProperty::Summary, event.title.as_str());
        if let Some(description) = &event.description {
            component.add_property(ICalendarProperty::Description, description.as_str());
        }
        if let Some(url) = &event.url {
            component.add_property(ICalendarProperty::Url, Uri::Location(url.clone()));
        }
        components.push(component);
    }

    components.insert(0, root);
    ICalendar { components }.to_string()
}

fn date_value_param() -> ICalendarParameter {
    ICalendarParameter::new(ICalendarParameterName::Value, ICalendarValueType::Date)
}

fn partial_date(date: NaiveDate) -> PartialDateTime {
    PartialDateTime {
        year: Some(date.year() as u16),
        month: Some(date.month() as u8),
        day: Some(date.day() as u8),
        ..Default::default()
    }
}
