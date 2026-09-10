r#"        DoweJustify.Center, DoweJustify.CenterSafe -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.CenterVertically)
        DoweJustify.End, DoweJustify.EndSafe -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Bottom)
        DoweJustify.Between -> Arrangement.SpaceBetween
        DoweJustify.Around -> Arrangement.SpaceAround
        DoweJustify.Evenly -> Arrangement.SpaceEvenly
        else -> Arrangement.spacedBy(gap ?: 0.dp, Alignment.Top)
    }

"#

