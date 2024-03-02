#!/usr/bin/env -S nr -!
(mapv #(str %1 \/ %2) (range 1 5) (range 7 11))
