#!/bin/python                                                               
# -*- coding: utf-8 -*-

n = int(input())
s = input()
print(int(s.count('I') == 1) if 'I' in s else s.count('A'))
