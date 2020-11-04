let timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone
document.cookie = `tangram-timezone=${timeZone};max-age=31536000`
