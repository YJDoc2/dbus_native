## This is helper code for converting raw byte array got as response into ascii-like string
l = [] # response byte array

# this will replace ascii numbers by actual characters, and for rest keep hex code, 
# however for some edge case, we use | and hex code
converted = "".join([chr(k) if chr(k).isascii() else "|" + hex(k) for k in l])
# for actually using this output, copy-pase above in editor and replace | by \




## The following code was being used for debug before the auth was implemented

# import socket
# l = socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
# l.connect('/run/user/1000/bus')
# l.send(b'\0')
# l.send(b'AUTH EXTERNAL 31303030\r\n')
# t = l.recv(1024)
# print(t)
# l.send(b'BEGIN\r\n')
# # msg = [b'l\1\0\1\0\0\0\0\1\0\0\0m\0\0\0\1\1o\0\25\0\0\0/org/freedesktop/DBus\0\0\0\3\1s\0\5\0\0\0Hello\0\0\0\6\1s\0\24\0\0\0org.freedesktop.DBus\0\0\0\0\2\1s\0\24\0\0\0org.freedesktop.DBus\0\0\0\0']
# msg = [b'l\1\0\1\0\0\0\0\1\0\0\0n\0\0\0\1\1o\0\25\0\0\0/org/freedesktop/DBus\0\0\0\6\1s\0\24\0\0\0org.freedesktop.DBus\0\0\0\0\2\1s\0\24\0\0\0org.freedesktop.DBus\0\0\0\0\3\1s\0\5\0\0\0Hello\0\0\0']
# l.sendmsg(msg)
# print(l.recvmsg(1024))
# print()
# print()
# msg = [b'l\1\0\19\0\0\0\2\0\0\0\240\0\0\0\1\1o\0\31\0\0\0/org/freedesktop/systemd1\0\0\0\0\0\0\0\3\1s\0\3\0\0\0Get\0\0\0\0\0\7\1s\0\7\0\0\0:1.1309\0\6\1s\0\30\0\0\0org.freedesktop.systemd1\0\0\0\0\0\0\0\0\2\1s\0\37\0\0\0org.freedesktop.DBus.Properties\0\10\1g\0\2ss\0 \0\0\0org.freedesktop.systemd1.Manager\0\0\0\0\f\0\0\0ControlGroup\0']
# l.sendmsg(msg)
# print(l.recvmsg(1024))