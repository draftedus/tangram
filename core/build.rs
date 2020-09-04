fn main() {
	cc::Build::new()
		.file("gbt/tree/bin_stats.c")
		.compile("bin_stats");
}
