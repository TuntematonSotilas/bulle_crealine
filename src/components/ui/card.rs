use leptos::prelude::*;
use leptos_ui::clx;

mod components {
    use super::*;
    clx! {Card, div, "bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm"}
    clx! {CardHeader, div, "grid auto-rows-min items-start gap-1.5 px-6", "[.border-b]:pb-6"}
    clx! {CardTitle, div, "font-semibold leading-none"}
    clx! {CardDescription, div, "text-sm text-muted-foreground"}
    clx! {CardContent, div, "px-6"}
    clx! {CardFooter, div, "flex items-center px-6", "[.border-t]:pt-6"}
}

pub use components::*;
