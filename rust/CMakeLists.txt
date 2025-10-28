cmake_minimum_required(VERSION 3.10)
project(nfqws C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Os -std=gnu99")

file(GLOB SRC_FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/zapret/nfq/*.c"
    "${CMAKE_CURRENT_SOURCE_DIR}/zapret/nfq/crypto/*.c"
)

add_library(nfqws STATIC ${SRC_FILES})

target_link_libraries(nfqws
    z
    netfilter_queue
    nfnetlink
    mnl
)
