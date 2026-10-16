#!/usr/bin/env bash
# Scripted conformance peer: reports that it cannot enforce Cursor's protected write.
set -eu
printf '%s\n' '{"host":"second-host-stub","capability":"protected-write","result":"unsupported"}'
exit 78
