#!/usr/bin/env Rscript
# Mbilinyi Tech Solutions — analytics reports.
#
# Reads a CSV of the live data on stdin and prints one JSON object on the last
# line of stdout. Base R only: no packages to install, so this runs on any host
# that has R itself.
#
#   Rscript report.R revenue < data.csv
#
# Every figure is computed here, from the rows that were actually sent. Nothing
# is estimated or filled in: an empty input produces zeros and empty groups
# rather than a plausible-looking number.

args <- commandArgs(trailingOnly = TRUE)
report <- if (length(args) >= 1) args[[1]] else ""

raw <- tryCatch(readLines(file("stdin"), warn = FALSE), error = function(e) character(0))

# A tiny JSON writer, so the output needs no package.
esc <- function(s) {
  s <- gsub("\\\\", "\\\\\\\\", s)
  s <- gsub('"', '\\\\"', s)
  s <- gsub("\n", "\\\\n", s)
  s <- gsub("\r", "", s)
  s <- gsub("\t", "\\\\t", s)
  s
}
q <- function(s) paste0('"', esc(as.character(s)), '"')
num <- function(x) {
  if (is.null(x) || length(x) == 0 || is.na(x)) return("0")
  if (is.infinite(x)) return("0")
  formatC(as.numeric(x), format = "f", digits = 2)
}
obj <- function(names, values) {
  parts <- mapply(function(n, v) paste0(q(n), ":", v), names, values, USE.NAMES = FALSE)
  paste0("{", paste(parts, collapse = ","), "}")
}
arr <- function(items) paste0("[", paste(items, collapse = ","), "]")
counts <- function(tab) {
  if (length(tab) == 0) return("{}")
  n <- names(tab)
  o <- order(n)
  obj(n[o], vapply(tab[o], function(x) formatC(as.integer(x), format = "d"), ""))
}

if (length(raw) < 2) {
  cat(paste0(
    '{"report":', q(report), ',"rows":0,"empty":true,',
    '"note":"No records yet, so there is nothing to summarise."}'
  ), "\n")
  quit(status = 0)
}

con <- textConnection(raw)
d <- tryCatch(
  read.csv(con, stringsAsFactors = FALSE, check.names = FALSE),
  error = function(e) NULL
)
close(con)
if (is.null(d) || nrow(d) == 0) {
  cat(paste0(
    '{"report":', q(report), ',"rows":0,"empty":true,',
    '"note":"The data could not be read."}'
  ), "\n")
  quit(status = 0)
}

n <- nrow(d)

if (report == "revenue") {
  amt <- suppressWarnings(as.numeric(d$amount))
  amt[is.na(amt)] <- 0
  paid <- sum(amt[d$status == "Paid"])
  unpaid <- sum(amt[d$status != "Paid"])
  by_month <- tapply(amt, d$month, sum)
  by_month[is.na(by_month)] <- 0
  months <- names(by_month)
  o <- order(months)
  month_json <- if (length(months) == 0) "[]" else {
    arr(mapply(
      function(m, v) paste0('{"month":', q(m), ',"total":', num(v), "}"),
      months[o], by_month[o],
      USE.NAMES = FALSE
    ))
  }
  cat(paste0(
    '{"report":"revenue","rows":', n,
    ',"paid":', num(paid),
    ',"unpaid":', num(unpaid),
    ',"collected_pct":', num(if (paid + unpaid > 0) 100 * paid / (paid + unpaid) else 0),
    ',"by_month":', month_json,
    ',"by_status":', counts(table(d$status)),
    "}\n"
  ))
  quit(status = 0)
}

if (report == "requests") {
  cat(paste0(
    '{"report":"requests","rows":', n,
    ',"by_service":', counts(table(d$service)),
    ',"by_status":', counts(table(d$status)),
    ',"by_priority":', counts(table(d$priority)),
    ',"open":', formatC(sum(d$status != "Completed" & d$status != "Rejected"), format = "d"),
    ',"completed":', formatC(sum(d$status == "Completed"), format = "d"),
    "}\n"
  ))
  quit(status = 0)
}

if (report == "payments") {
  amt <- suppressWarnings(as.numeric(d$amount))
  amt[is.na(amt)] <- 0
  settled <- d$status == "Settled"
  by_gateway <- tapply(amt, d$gateway, sum)
  by_gateway[is.na(by_gateway)] <- 0
  g <- names(by_gateway)
  o <- order(-by_gateway)
  gw_json <- if (length(g) == 0) "[]" else {
    arr(mapply(
      function(k, v) {
        rows <- d[d$gateway == k, ]
        rate <- if (nrow(rows) > 0) 100 * sum(rows$status == "Settled") / nrow(rows) else 0
        paste0(
          '{"gateway":', q(k), ',"volume":', num(v),
          ',"attempts":', formatC(nrow(rows), format = "d"),
          ',"settled":', formatC(sum(rows$status == "Settled"), format = "d"),
          ',"settlement_pct":', num(rate), "}"
        )
      },
      g[o], by_gateway[o],
      USE.NAMES = FALSE
    ))
  }
  cat(paste0(
    '{"report":"payments","rows":', n,
    ',"attempts":', formatC(n, format = "d"),
    ',"settled":', formatC(sum(settled), format = "d"),
    ',"pending":', formatC(sum(d$status == "Pending"), format = "d"),
    ',"failed":', formatC(sum(d$status == "Failed"), format = "d"),
    ',"settled_value":', num(sum(amt[settled])),
    ',"by_gateway":', gw_json,
    ',"by_currency":', counts(table(d$currency)),
    "}\n"
  ))
  quit(status = 0)
}

if (report == "escrow") {
  amt <- suppressWarnings(as.numeric(d$amount))
  amt[is.na(amt)] <- 0
  holds <- tapply(amt[d$kind == "hold"], d$contract_id[d$kind == "hold"], sum)
  outs <- tapply(amt[d$kind %in% c("release", "refund")], d$contract_id[d$kind %in% c("release", "refund")], sum)
  ids <- unique(d$contract_id)
  rows <- lapply(ids, function(id) {
    h <- if (id %in% names(holds)) as.numeric(holds[[id]]) else 0
    r <- if (id %in% names(outs)) as.numeric(outs[[id]]) else 0
    list(id = id, held = h, released = r, balance = h - r)
  })
  # Largest balance first, so the biggest exposure is at the top.
  ord <- order(-vapply(rows, function(x) x$balance, 0))
  rows <- rows[ord]
  body <- arr(lapply(rows, function(x) {
    paste0(
      '{"contract":', q(x$id), ',"held":', num(x$held),
      ',"released":', num(x$released), ',"balance":', num(x$balance), "}"
    )
  }))
  total_held <- sum(vapply(rows, function(x) x$held, 0))
  total_out <- sum(vapply(rows, function(x) x$released, 0))
  cat(paste0(
    '{"report":"escrow","rows":', n,
    ',"contracts":', formatC(length(ids), format = "d"),
    ',"total_held":', num(total_held),
    ',"total_released":', num(total_out),
    ',"outstanding":', num(total_held - total_out),
    ',"by_contract":', body,
    "}\n"
  ))
  quit(status = 0)
}

cat(paste0('{"report":', q(report), ',"error":"Unknown report."}'), "\n")
