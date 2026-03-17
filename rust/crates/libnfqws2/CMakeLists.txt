cmake_minimum_required(VERSION 3.10)
project(nfqws2 C)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

file(GLOB NFQ_SOURCES "zapret2/nfq2/*.c")
file(GLOB NFQ_CRYPTO_SOURCES "zapret2/nfq2/crypto/*.c")

add_library(nfqws2 SHARED ${NFQ_SOURCES} ${NFQ_CRYPTO_SOURCES})

target_include_directories(nfqws2 PRIVATE
    "zapret2/nfq2"
    "zapret2/nfq2/crypto"
    $ENV{NETFILTER_LIBS}/include
    $ENV{LUAJIT_LIBS}/include/luajit-2.1
)

target_compile_definitions(nfqws2 PRIVATE main=nfqws2_main)
target_compile_options(nfqws2 PRIVATE -w)

set_target_properties(nfqws2 PROPERTIES LINK_FLAGS "-Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/nfqws2.map")

if(ANDROID)
    find_library(LOG_LIB log)
    target_link_libraries(nfqws2 PRIVATE ${LOG_LIB})
endif()

target_link_libraries(nfqws2 PRIVATE
    $ENV{NETFILTER_LIBS}/lib/libnetfilter_queue.a
    $ENV{NETFILTER_LIBS}/lib/libnfnetlink.a
    $ENV{NETFILTER_LIBS}/lib/libmnl.a
    $ENV{LUAJIT_LIBS}/lib/libluajit-5.1.a
    z
    unwind
)

install(TARGETS nfqws2
    LIBRARY DESTINATION lib
)