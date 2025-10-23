# Homoglyph attack using Cyrillic 'а' (U+0430) instead of Latin 'a' (U+0061)
def check_аdmin():  # First 'a' is Cyrillic
    return True

if check_admin():  # This 'a' is Latin - different function!
    print("Access granted")
