//! Month and weekday names. fluent_ui takes these from Flutter's `intl`
//! (CLDR data), which its ARB files do not carry, so they live here for the
//! common locales. Others fall back to English.

use crate::date::Weekday;

/// Calendar names for one locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateNames {
    pub locale: &'static str,
    /// Stand-alone month names, January first.
    pub months: [&'static str; 12],
    /// Short weekday names for calendar headers, Sunday first.
    pub weekdays: [&'static str; 7],
    /// The first column of a calendar month.
    pub first_day_of_week: Weekday,
}

use Weekday::{Monday, Saturday, Sunday};

macro_rules! names {
    ($locale:literal, [$($m:literal),*], [$($w:literal),*], $first:expr) => {
        DateNames { locale: $locale, months: [$($m),*], weekdays: [$($w),*], first_day_of_week: $first }
    };
}

#[rustfmt::skip]
pub static DATE_NAMES: &[DateNames] = &[
    names!("en", ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
        ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"], Sunday),
    names!("ru", ["Январь", "Февраль", "Март", "Апрель", "Май", "Июнь", "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь"],
        ["Вс", "Пн", "Вт", "Ср", "Чт", "Пт", "Сб"], Monday),
    names!("uk", ["Січень", "Лютий", "Березень", "Квітень", "Травень", "Червень", "Липень", "Серпень", "Вересень", "Жовтень", "Листопад", "Грудень"],
        ["Нд", "Пн", "Вт", "Ср", "Чт", "Пт", "Сб"], Monday),
    names!("be", ["Студзень", "Люты", "Сакавік", "Красавік", "Май", "Чэрвень", "Ліпень", "Жнівень", "Верасень", "Кастрычнік", "Лістапад", "Снежань"],
        ["Нд", "Пн", "Аў", "Ср", "Чц", "Пт", "Сб"], Monday),
    names!("de", ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"],
        ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"], Monday),
    names!("fr", ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"],
        ["Di", "Lu", "Ma", "Me", "Je", "Ve", "Sa"], Monday),
    names!("es", ["Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio", "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"],
        ["Do", "Lu", "Ma", "Mi", "Ju", "Vi", "Sá"], Monday),
    names!("it", ["Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno", "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre"],
        ["Do", "Lu", "Ma", "Me", "Gi", "Ve", "Sa"], Monday),
    names!("pt", ["Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho", "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro"],
        ["Do", "Se", "Te", "Qa", "Qi", "Sx", "Sá"], Sunday),
    names!("nl", ["Januari", "Februari", "Maart", "April", "Mei", "Juni", "Juli", "Augustus", "September", "Oktober", "November", "December"],
        ["Zo", "Ma", "Di", "Wo", "Do", "Vr", "Za"], Monday),
    names!("pl", ["Styczeń", "Luty", "Marzec", "Kwiecień", "Maj", "Czerwiec", "Lipiec", "Sierpień", "Wrzesień", "Październik", "Listopad", "Grudzień"],
        ["Nd", "Pn", "Wt", "Śr", "Cz", "Pt", "So"], Monday),
    names!("cs", ["Leden", "Únor", "Březen", "Duben", "Květen", "Červen", "Červenec", "Srpen", "Září", "Říjen", "Listopad", "Prosinec"],
        ["Ne", "Po", "Út", "St", "Čt", "Pá", "So"], Monday),
    names!("sk", ["Január", "Február", "Marec", "Apríl", "Máj", "Jún", "Júl", "August", "September", "Október", "November", "December"],
        ["Ne", "Po", "Ut", "St", "Št", "Pi", "So"], Monday),
    names!("sv", ["Januari", "Februari", "Mars", "April", "Maj", "Juni", "Juli", "Augusti", "September", "Oktober", "November", "December"],
        ["Sö", "Må", "Ti", "On", "To", "Fr", "Lö"], Monday),
    names!("ro", ["Ianuarie", "Februarie", "Martie", "Aprilie", "Mai", "Iunie", "Iulie", "August", "Septembrie", "Octombrie", "Noiembrie", "Decembrie"],
        ["Du", "Lu", "Ma", "Mi", "Jo", "Vi", "Sâ"], Monday),
    names!("hu", ["Január", "Február", "Március", "Április", "Május", "Június", "Július", "Augusztus", "Szeptember", "Október", "November", "December"],
        ["V", "H", "K", "Sze", "Cs", "P", "Szo"], Monday),
    names!("hr", ["Siječanj", "Veljača", "Ožujak", "Travanj", "Svibanj", "Lipanj", "Srpanj", "Kolovoz", "Rujan", "Listopad", "Studeni", "Prosinac"],
        ["Ne", "Po", "Ut", "Sr", "Če", "Pe", "Su"], Monday),
    names!("ca", ["Gener", "Febrer", "Març", "Abril", "Maig", "Juny", "Juliol", "Agost", "Setembre", "Octubre", "Novembre", "Desembre"],
        ["Dg", "Dl", "Dt", "Dc", "Dj", "Dv", "Ds"], Monday),
    names!("tr", ["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"],
        ["Pz", "Pt", "Sa", "Ça", "Pe", "Cu", "Ct"], Monday),
    names!("el", ["Ιανουάριος", "Φεβρουάριος", "Μάρτιος", "Απρίλιος", "Μάιος", "Ιούνιος", "Ιούλιος", "Αύγουστος", "Σεπτέμβριος", "Οκτώβριος", "Νοέμβριος", "Δεκέμβριος"],
        ["Κυ", "Δε", "Τρ", "Τε", "Πε", "Πα", "Σά"], Monday),
    names!("id", ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"],
        ["Mg", "Sn", "Sl", "Rb", "Km", "Jm", "Sb"], Monday),
    names!("ms", ["Januari", "Februari", "Mac", "April", "Mei", "Jun", "Julai", "Ogos", "September", "Oktober", "November", "Disember"],
        ["Ah", "Is", "Se", "Ra", "Kh", "Ju", "Sa"], Monday),
    names!("vi", ["Tháng 1", "Tháng 2", "Tháng 3", "Tháng 4", "Tháng 5", "Tháng 6", "Tháng 7", "Tháng 8", "Tháng 9", "Tháng 10", "Tháng 11", "Tháng 12"],
        ["CN", "T2", "T3", "T4", "T5", "T6", "T7"], Monday),
    names!("zh", ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
        ["日", "一", "二", "三", "四", "五", "六"], Monday),
    names!("zh_Hant", ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
        ["日", "一", "二", "三", "四", "五", "六"], Sunday),
    names!("ja", ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
        ["日", "月", "火", "水", "木", "金", "土"], Sunday),
    names!("ko", ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"],
        ["일", "월", "화", "수", "목", "금", "토"], Sunday),
    names!("he", ["ינואר", "פברואר", "מרץ", "אפריל", "מאי", "יוני", "יולי", "אוגוסט", "ספטמבר", "אוקטובר", "נובמבר", "דצמבר"],
        ["א׳", "ב׳", "ג׳", "ד׳", "ה׳", "ו׳", "ש׳"], Sunday),
    names!("ar", ["يناير", "فبراير", "مارس", "أبريل", "مايو", "يونيو", "يوليو", "أغسطس", "سبتمبر", "أكتوبر", "نوفمبر", "ديسمبر"],
        ["ح", "ن", "ث", "ر", "خ", "ج", "س"], Saturday),
];

impl DateNames {
    /// The names for a locale tag in the form [`super::Localizations`] uses,
    /// falling back to the language alone and then to English.
    pub fn for_locale(tag: &str) -> &'static Self {
        let tag = tag.replace('-', "_");
        let language = tag.split('_').next().unwrap_or_default();
        DATE_NAMES
            .iter()
            .find(|n| n.locale.eq_ignore_ascii_case(&tag))
            .or_else(|| {
                DATE_NAMES
                    .iter()
                    .find(|n| n.locale.eq_ignore_ascii_case(language))
            })
            .unwrap_or(&DATE_NAMES[0])
    }

    pub fn month(&self, month: u8) -> &'static str {
        self.months[usize::from(month.clamp(1, 12) - 1)]
    }

    /// Up to three characters of the month name, for the year view.
    pub fn month_short(&self, month: u8) -> String {
        self.month(month).chars().take(3).collect()
    }

    pub fn weekday(&self, day: Weekday) -> &'static str {
        self.weekdays[usize::from(day.index())]
    }
}
