cmake_minimum_required(VERSION 3.10)
project(nfqws C)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

file(GLOB NFQ_SOURCES "zapret/nfq/*.c")
file(GLOB NFQ_CRYPTO_SOURCES "zapret/nfq/crypto/*.c")

add_library(nfqws SHARED
    ${NFQ_SOURCES}
    ${NFQ_CRYPTO_SOURCES}
)

target_include_directories(nfqws PRIVATE
    zapret/nfq
    zapret/nfq/crypto
)

target_compile_definitions(nfqws PRIVATE main=nfqws_main)
target_compile_options(nfqws PRIVATE -w)

set_target_properties(nfqws PROPERTIES LINK_FLAGS "-Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/nfqws.map")

if(ANDROID)
    find_library(LOG_LIB log)
    target_link_libraries(nfqws PRIVATE ${LOG_LIB})
endif()

target_link_libraries(nfqws PRIVATE
    $ENV{NETFILTER_LIBS}/lib/libnetfilter_queue.a
    $ENV{NETFILTER_LIBS}/lib/libnfnetlink.a
    $ENV{NETFILTER_LIBS}/lib/libmnl.a
    z
    unwind
)

install(TARGETS nfqws
    LIBRARY DESTINATION lib
)