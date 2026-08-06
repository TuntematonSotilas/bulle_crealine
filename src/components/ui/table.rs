use leptos::prelude::*;
use leptos_ui::clx;

mod components {
    use super::*;
    clx! {TableContainer, div, "relative w-full overflow-x-auto"}
    clx! {Table, table, "w-full caption-bottom text-sm border-collapse"}
    clx! {TableHeader, thead, "[&_tr]:border-b"}
    clx! {TableBody, tbody, "[&_tr:last-child]:border-0"}
    clx! {TableRow, tr, "border-b transition-colors hover:bg-muted/50"}
    clx! {TableHead, th, "h-10 px-2 text-left align-middle font-medium text-muted-foreground whitespace-nowrap"}
    clx! {TableCell, td, "p-2 align-middle"}
    clx! {TableCaption, caption, "mt-4 text-sm text-muted-foreground"}
}

pub use components::*;
