#pragma once

#if defined(_MSC_VER)
//  Microsoft
#define NLC__EXPORT __declspec(dllexport)
#define NLC__IMPORT __declspec(dllimport)
#elif defined(__GNUC__)
//  GCC
#define NLC__EXPORT __attribute__((visibility("default")))
#define NLC__IMPORT
#else
//  do nothing and hope for the best?
#define NLC__EXPORT
#define NLC__IMPORT
#pragma error Unknown dynamic link import / export semantics.
#endif
