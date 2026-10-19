#!/usr/bin/env python3
"""Corpus propio MIT, determinista, sin datos personales ni archivos descargados.

No usa lopdf: escribe objetos/xref y calcula golden desde strings explícitos.
Los PDF binarios se generan sólo en memoria o en el directorio de evidencia.
"""
import hashlib
import base64
import json
import zlib

SEED = "noteka-pdf-v1-2026-10-13"  # Identidad reproducible; no usa azar/fechas/UUID.
CAP = 1024 * 1024


def stream(data, compressed=False, extra=b""):
    if compressed:
        data = zlib.compress(data, 9)
        extra += b" /Filter /FlateDecode"
    return b"<< /Length " + str(len(data)).encode() + extra + b" >>\nstream\n" + data + b"\nendstream"


def serialize(objects, trailer=b"", version=b"1.7", padding=0):
    output = b"%PDF-" + version + b"\n%\xe2\xe3\xcf\xd3\n"
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(output))
        output += f"{index} 0 obj\n".encode() + obj + b"\nendobj\n"
    if padding:
        assert padding >= 2
        output += b"%" + b"p" * (padding - 2) + b"\n"
    start = len(output)
    output += f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode()
    output += b"".join(f"{offset:010d} 00000 n \n".encode() for offset in offsets[1:])
    output += (f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R ".encode() + trailer
               + f">>\nstartxref\n{start}\n%%EOF\n".encode())
    return output


FONT = b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /StandardEncoding >>"
RESOURCES = b"<< /Font << /F1 3 0 R >> /XObject << /Im 4 0 R >> >>"
IMAGE = stream(b"\xff\xff\xff", extra=b" /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8")


def document(contents, font=FONT, compressed=False, page_extra=b"", root_extra=b"", resources=RESOURCES):
    objects = [b"<< /Type /Catalog /Pages 2 0 R " + root_extra + b" >>", b"", font, IMAGE]
    kids = []
    for content in contents:
        page_id = len(objects) + 1
        kids.append(f"{page_id} 0 R".encode())
        objects += [b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] "
                    + f"/Contents {page_id + 1} 0 R ".encode() + page_extra + b" >>",
                    stream(content, compressed)]
    objects[1] = (b"<< /Type /Pages /Count " + str(len(kids)).encode() + b" /Kids ["
                  + b" ".join(kids) + b"] /Resources " + resources + b" >>")
    return objects


def rc4(key, data):
    state = list(range(256))
    j = 0
    for i in range(256):
        j = (j + state[i] + key[i % len(key)]) % 256
        state[i], state[j] = state[j], state[i]
    i = j = 0
    result = bytearray()
    for byte in data:
        i = (i + 1) % 256
        j = (j + state[i]) % 256
        state[i], state[j] = state[j], state[i]
        result.append(byte ^ state[(state[i] + state[j]) % 256])
    return bytes(result)


def encrypted(password):
    # Standard Security Handler revision 2, algoritmos 2–5 ISO 32000-1 §7.6.
    # Página sin streams/strings: sólo O/U necesitan cifrado. Sin credenciales reales.
    pad = bytes.fromhex("28bf4e5e4e758a4164004e56fffa01082e2e00b6d0683e802f0ca9fe6453697a")
    padded = (password + pad)[:32]
    owner = rc4(hashlib.md5((b"synthetic-owner" + pad)[:32]).digest()[:5], padded)
    identifier = hashlib.md5(SEED.encode()).digest()
    permissions = (-4).to_bytes(4, "little", signed=True)
    key = hashlib.md5(padded + owner + permissions + identifier).digest()[:5]
    user = rc4(key, pad)
    objects = [b"<< /Type /Catalog /Pages 2 0 R >>",
               b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
               b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] >>",
               b"<< /Filter /Standard /V 1 /R 2 /Length 40 /P -4 /O <" + owner.hex().encode()
               + b"> /U <" + user.hex().encode() + b"> >>"]
    return serialize(objects, b"/Encrypt 4 0 R /ID [<" + identifier.hex().encode() + b"><" + identifier.hex().encode() + b">] ")


def xref_stream(objects, body=None, extra=b"", compressed=False, compressed_entries=None, transform=None):
    output = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n"
    offsets = []
    for index, obj in enumerate(objects, 1):
        if obj is None:
            offsets.append(None)
            continue
        offsets.append(len(output))
        output += f"{index} 0 obj\n".encode() + obj + b"\nendobj\n"
    start = len(output)
    xref_id = len(objects) + 1
    offsets.append(start)
    if body is None:
        body = b"\0\0\0\0\0\xff\xff"
        for index, offset in enumerate(offsets, 1):
            if compressed_entries and index in compressed_entries:
                container, member = compressed_entries[index]
                body += b"\2" + container.to_bytes(4, "big") + member.to_bytes(2, "big")
            else:
                assert offset is not None
                body += b"\1" + offset.to_bytes(4, "big") + b"\0\0"
    if transform:
        body = transform(body)
    dict_extra = f" /Type /XRef /Root 1 0 R /Size {xref_id + 1} /W [1 4 2] ".encode() + extra
    output += f"{xref_id} 0 obj\n".encode() + stream(body, compressed, dict_extra) + b"\nendobj\n"
    return output + f"startxref\n{start}\n%%EOF\n".encode()


def startxref(source):
    return int(source.rsplit(b"startxref\n", 1)[1].splitlines()[0])


def revision(source, replacements, size=7, extra=b"", free=()):
    """Append propio, sin usar escritor/resultado de una librería PDF."""
    previous = startxref(source)
    output = source
    entries = {}
    for object_id, content in replacements:
        entries[object_id] = (len(output), b"n")
        output += f"{object_id} 0 obj\n".encode() + content + b"\nendobj\n"
    for object_id in free:
        entries[object_id] = (0, b"f")
    xref = len(output)
    output += b"xref\n"
    for object_id, (offset, kind) in sorted(entries.items()):
        output += f"{object_id} 1\n{offset:010d} 00000 ".encode() + kind + b" \n"
    output += f"trailer\n<< /Size {size} /Root 1 0 R /Prev {previous} ".encode() + extra + b" >>\n"
    return output + f"startxref\n{xref}\n%%EOF\n".encode()


def hybrid_replacement(source, content):
    """La tabla conserva el offset viejo; el suplemento debe reemplazarlo."""
    previous = startxref(source)
    old = source.index(b"6 0 obj\n")
    replacement = len(source)
    output = source + b"6 0 obj\n" + stream(content) + b"\nendobj\n"
    supplement = len(output)
    rows = b"\1" + replacement.to_bytes(4,'big') + b"\0\0" + b"\1" + supplement.to_bytes(4,'big') + b"\0\0"
    output += b"7 0 obj\n" + stream(rows,extra=b" /Type /XRef /Size 8 /Index [6 2] /W [1 4 2]") + b"\nendobj\n"
    main = len(output)
    output += f"xref\n6 2\n{old:010} 00000 n \n{supplement:010} 00000 n \ntrailer\n<< /Size 8 /Root 1 0 R /Prev {previous} /XRefStm {supplement} >>\nstartxref\n{main}\n%%EOF\n".encode()
    return output


def lzw_codes(codes, early=1):
    """Packing de códigos explícitos; no implementa búsqueda/diccionario de strings."""
    output = []
    width, next_code, has_previous = 9, 258, False
    for code in codes:
        output.append(f"{code:0{width}b}")
        if code == 256:
            width, next_code, has_previous = 9, 258, False
        elif code != 257:
            if has_previous and next_code < 4096:
                next_code += 1
                if width < 12 and next_code + early == 2 ** width:
                    width += 1
            has_previous = True
    bits = "".join(output)
    bits += "0" * (-len(bits) % 8)
    return int(bits, 2).to_bytes(len(bits) // 8, "big")


def linearized_framing():
    """Estructura linealizada propia: hints no usados para lectura completa."""
    prefix = b"%PDF-1.7\n7 0 obj\n<< /Linearized 1 /L 0000000000 /H [0000000000 0000000000] /O 5 /E 0000000000 /N 1 /T 0000000000 >>\nendobj\n"
    first = len(prefix)
    output = prefix + (f"xref\n7 1\n0000000009 00000 n \ntrailer\n<< /Size 9 /Root 1 0 R /Prev 0000000000 >>\nstartxref\n0\n%%EOF\n").encode()
    offsets = []
    for number, obj in enumerate(document([b"BT /F1 12 Tf (Linear) Tj ET\n"]), 1):
        offsets.append(len(output))
        output += f"{number} 0 obj\n".encode() + obj + b"\nendobj\n"
    end_page = len(output)
    hint_offset = len(output)
    output += b"8 0 obj\n" + stream(bytes(36), extra=b" /S 36") + b"\nendobj\n"
    hint_length = len(output) - hint_offset
    main = len(output)
    output += b"xref\n0 7\n0000000000 65535 f \n" + b"".join(f"{n:010} 00000 n \n".encode() for n in offsets)
    output += f"8 1\n{hint_offset:010} 00000 n \ntrailer\n<< /Size 9 >>\nstartxref\n{first}\n%%EOF\n".encode()
    for key, val in [(b"L", len(output)), (b"E", end_page), (b"T", main + len(b"xref\n0 7\n")), (b"Prev", main)]:
        output = output.replace(b"/" + key + b" 0000000000", b"/" + key + f" {val:010}".encode(), 1)
    output = output.replace(b"/H [0000000000 0000000000]", f"/H [{hint_offset:010} {hint_length:010}]".encode())
    return output


def tiff_encode(data, colors, bits):
    count = len(data) * 8 // bits
    samples = [(int.from_bytes(data, "big") >> ((count - 1 - i) * bits)) & ((1 << bits) - 1) for i in range(count)]
    filtered = [sample if i < colors else (sample - samples[i - colors]) % (1 << bits) for i, sample in enumerate(samples)]
    packed = 0
    for sample in filtered:
        packed = (packed << bits) | sample
    return packed.to_bytes(len(data), "big")


def png_encode(data, columns, filters=(0, 1, 2, 3, 4)):
    assert len(data) % columns == 0
    output = bytearray()
    previous = bytes(columns)
    for number, start in enumerate(range(0, len(data), columns)):
        row = data[start:start + columns]
        algorithm = filters[number % len(filters)]
        output.append(algorithm)
        for i, byte in enumerate(row):
            left = row[i - 1] if i else 0
            up = previous[i]
            upper_left = previous[i - 1] if i else 0
            target = left + up - upper_left
            paeth = min([(abs(target - left), 0, left), (abs(target - up), 1, up), (abs(target - upper_left), 2, upper_left)])[2]
            prediction = [0, left, up, (left + up) // 2, paeth][algorithm]
            output.append((byte - prediction) % 256)
        previous = row
    return bytes(output)


def cases():
    result = []

    def add(name, objects=None, texts=None, error=None, raw=None, **kwargs):
        assert (texts is None) != (error is None), name
        result.append({"name": name, "source": raw if raw is not None else serialize(objects, **kwargs),
                       "texts": texts, "error": error})

    plain = b"BT /F1 12 Tf (Hello PDF) Tj ET\n"
    for minor in range(8):
        add(f"version-1-{minor}", document([plain]), ["Hello PDF\n"], version=f"1.{minor}".encode())
    add("all-show-operators", document([b'BT /F1 12 Tf (Uno) Tj [( ) -120 (dos)] TJ (tres) \' 1 2 (cuatro) " ET\n']), ["Uno dos\ntres\ncuatro\n"])
    add("winansi", document([b"BT /F1 12 Tf <e120e920f120802093> Tj ET\n"], font=FONT.replace(b"StandardEncoding", b"WinAnsiEncoding")), ["á é ñ € “\n"])
    add("standard-quotes-ligature", document([b"BT /F1 12 Tf <276020a120ae> Tj ET\n"]), ["’‘ ¡ ﬁ\n"])
    add("absent-encoding", document([plain], font=FONT.replace(b" /Encoding /StandardEncoding", b"")), ["Hello PDF\n"])
    for basefont in [b"Courier", b"Courier-Bold", b"Courier-Oblique", b"Courier-BoldOblique",
                     b"Helvetica", b"Helvetica-Bold", b"Helvetica-Oblique", b"Helvetica-BoldOblique",
                     b"Times-Roman", b"Times-Bold", b"Times-Italic", b"Times-BoldItalic"]:
        add("font-" + basefont.decode(), document([plain], font=FONT.replace(b"Helvetica", basefont)), ["Hello PDF\n"])
    add("empty-image-mixed-order", document([b"", b"BT ET\n", b"/Im Do\n", plain + b"/Im Do\n", b"BT /F1 12 Tf (Last) Tj ET\n"]), ["", "", "", "Hello PDF\n", "Last\n"])
    add("geometry-ignored", document([b"q 1 0 0 1 0 0 cm BT /F1 12 Tf (B) Tj 1 0 0 1 0 500 Tm (A) Tj ET Q\n"]), ["BA\n"])
    add("text-line-operators", document([b"BT /F1 12 Tf (A) Tj 1 0 Td (B) Tj 0 -12 TD (C) Tj T* (D) Tj ET\n"]), ["AB\nC\nD\n"])
    add("marked-content", document([b"/Span << /MCID 0 >> BDC " + plain + b"EMC\n"]), ["Hello PDF\n"])
    add("flate", document([plain], compressed=True), ["Hello PDF\n"])
    def encoded_case(name, data, extra, error=None, texts=None):
        objects = document([plain])
        objects[-1] = stream(data, extra=extra)
        add(name, objects, texts=(["Hello PDF\n"] if texts is None and error is None else texts), error=error)

    encoded_case("asciihex", plain.hex().encode() + b">", b" /Filter /ASCIIHexDecode")
    encoded_case("ascii85", base64.a85encode(plain) + b"~>", b" /Filter /ASCII85Decode")
    encoded_case("runlength", bytes([len(plain) - 1]) + plain + b"\x80", b" /Filter /RunLengthDecode")
    for early in [0, 1]:
        encoded_case(f"lzw-{early}", lzw_codes([256] + list(plain) + [257], early), b" /Filter /LZWDecode /DecodeParms << /EarlyChange " + str(early).encode() + b" >>")
        long = b"BT /F1 12 Tf (" + b"x" * 9000 + b") Tj ET\n"
        encoded_case(f"lzw-width-transitions-{early}", lzw_codes([256] + list(long) + [257], early),
                     b" /Filter /LZWDecode /DecodeParms << /EarlyChange " + str(early).encode() + b" >>", texts=["x" * 9000 + "\n"])
    encoded_case("filter-chain-ascii85-flate", base64.a85encode(zlib.compress(plain, 9)) + b"~>",
                 b" /Filter [/ASCII85Decode /FlateDecode] /DecodeParms [null null]")
    for bits in [1, 2, 4, 8, 16]:
        padded = plain + b" " * (len(plain) % 2 if bits == 16 else 0)
        filtered = tiff_encode(padded, 1, bits)
        encoded_case(f"tiff-{bits}", zlib.compress(filtered, 9), f" /Filter /FlateDecode /DecodeParms << /Predictor 2 /BitsPerComponent {bits} /Columns {len(padded) * 8 // bits} >>".encode())
    for filter_number in range(5):
        padded = plain + b" " * (-len(plain) % 7)
        encoded_case(f"png-fixed-{10 + filter_number}", zlib.compress(png_encode(padded, 7, (filter_number,)), 9),
                     f" /Filter /FlateDecode /DecodeParms << /Predictor {10 + filter_number} /Columns 7 >>".encode())
    encoded_case("png-adaptive", zlib.compress(png_encode(padded, 7), 9), b" /Filter /FlateDecode /DecodeParms << /Predictor 15 /Columns 7 >>")
    for declared in range(10, 15):
        encoded_case(f"png-row-tags-override-{declared}", zlib.compress(png_encode(padded, 7), 9),
                     f" /Filter /FlateDecode /DecodeParms << /Predictor {declared} /Columns 7 >>".encode())
    add("xref-png-up", raw=xref_stream(document([plain]), compressed=True,
        extra=b"/DecodeParms << /Predictor 12 /Columns 7 >>", transform=lambda body: png_encode(body, 7, (2,))), texts=["Hello PDF\n"])
    for name, data, extra, error in [
        ("hex-missing-eod", b"61", b"/ASCIIHexDecode", "pdf_decode_error"),
        ("hex-trailing", b"61>junk", b"/ASCIIHexDecode", "pdf_decode_error"),
        ("hex-invalid", b"6G>", b"/ASCIIHexDecode", "pdf_decode_error"),
        ("85-missing-eod", b"z", b"/ASCII85Decode", "pdf_decode_error"),
        ("85-invalid-z", b"!z~>", b"/ASCII85Decode", "pdf_decode_error"),
        ("85-overflow", b"uuuuu~>", b"/ASCII85Decode", "pdf_decode_error"),
        ("runlength-truncated", b"\x02ab", b"/RunLengthDecode", "pdf_decode_error"),
        ("runlength-missing-eod", b"\x00a", b"/RunLengthDecode", "pdf_decode_error"),
        ("lzw-missing-clear", lzw_codes([65, 257]), b"/LZWDecode", "pdf_decode_error"),
        ("lzw-invalid-code", lzw_codes([256, 300, 257]), b"/LZWDecode", "pdf_decode_error"),
        ("lzw-missing-eod", lzw_codes([256, 65]), b"/LZWDecode", "pdf_decode_error"),
    ]:
        encoded_case(name, data, b" /Filter " + extra, error=error)
    for name, data, params, error in [
        ("predictor-unknown", plain, b"/Predictor 3", "pdf_unsupported_feature"),
        ("predictor-zero-columns", plain, b"/Predictor 2 /Columns 0", "invalid_pdf"),
        ("predictor-zero-colors", plain, b"/Predictor 2 /Colors 0", "invalid_pdf"),
        ("predictor-invalid-bits", plain, b"/Predictor 2 /BitsPerComponent 3", "invalid_pdf"),
        ("predictor-row-truncated", b"\x02abcd", b"/Predictor 12 /Columns 7", "pdf_decode_error"),
        ("predictor-bad-filter", b"\x05abcdefg", b"/Predictor 15 /Columns 7", "pdf_decode_error"),
        ("predictor-overflow", plain, b"/Predictor 2 /Colors 9223372036854775807 /Columns 9223372036854775807", "pdf_stream_limit"),
    ]:
        encoded_case(name, zlib.compress(data, 9), b" /Filter /FlateDecode /DecodeParms << " + params + b" >>", error=error)
    encoded_case("lzw-kwkwk-bomb", lzw_codes([256, 32] + list(range(258, 3800)) + [257]), b" /Filter /LZWDecode", error="pdf_stream_limit")
    encoded_case("runlength-bomb", b"\x81 " * (4 * CAP // 128) + b"\x00 \x80", b" /Filter /RunLengthDecode", error="pdf_stream_limit")
    for compressed in [False, True]:
        add(f"xref-stream-{compressed}", raw=xref_stream(document([plain]), compressed=compressed), texts=["Hello PDF\n"])
    # Un miembro ObjStm no activo se valida, pero no habilita referencias a él.
    objects = document([plain]) + [stream(b"8 0 << /Synthetic true >>", True, b" /Type /ObjStm /N 1 /First 4")]
    add("object-stream", objects, ["Hello PDF\n"])
    add("object-stream-indexed-member", raw=xref_stream(objects + [None], compressed_entries={8: (7, 0)}), texts=["Hello PDF\n"])
    compressed_font = document([plain])
    compressed_font[2] = None
    compressed_font.append(stream(b"3 0 " + FONT, True, b" /Type /ObjStm /N 1 /First 4"))
    add("compressed-font", raw=xref_stream(compressed_font, compressed_entries={3: (7, 0)}), texts=["Hello PDF\n"])
    add("compressed-wrong-index", raw=xref_stream(compressed_font, compressed_entries={3: (7, 1)}), error="invalid_pdf")
    unindexed_font = document([plain], resources=RESOURCES.replace(b"3 0 R", b"42 0 R"))
    unindexed_font.append(stream(b"42 0 " + FONT, extra=b" /Type /ObjStm /N 1 /First 5"))
    add("compressed-member-absent-xref", raw=xref_stream(unindexed_font), error="invalid_pdf")
    for bits in [1, 2, 4, 8, 16]:
        member = b"3 0 " + FONT
        member += b" " * (len(member) % 2 if bits == 16 else 0)
        predicted = compressed_font.copy()
        predicted[-1] = stream(zlib.compress(tiff_encode(member, 1, bits), 9),
            extra=f" /Type /ObjStm /N 1 /First 4 /Filter /FlateDecode /DecodeParms << /Predictor 2 /BitsPerComponent {bits} /Columns {len(member) * 8 // bits} >>".encode())
        add(f"compressed-font-tiff-{bits}", raw=xref_stream(predicted, compressed_entries={3: (7, 0)}), texts=["Hello PDF\n"])
    lost = objects.copy()
    lost[-1] = stream(b"8 0 garbage", True, b" /Type /ObjStm /N 1 /First 4")
    add("object-stream-silent-loss", lost, error="invalid_pdf")
    duplicate = objects.copy()
    duplicate[-1] = stream(b"8 0 8 4 null null", True, b" /Type /ObjStm /N 2 /First 8")
    add("object-stream-duplicate", duplicate, error="invalid_pdf")
    for cap in [CAP, CAP + 1]:
        bomb = document([plain]) + [stream(b"8 0 " + b" " * (cap - 8) + b"null", True, b" /Type /ObjStm /N 1 /First 4")]
        add(f"object-stream-{cap}", bomb, texts=["Hello PDF\n"] if cap == CAP else None,
            error="pdf_stream_limit" if cap > CAP else None)
        add(f"object-stream-indexed-{cap}",raw=xref_stream(bomb+[None],compressed_entries={8:(7,0)}),texts=["Hello PDF\n"] if cap==CAP else None,error="pdf_stream_limit" if cap>CAP else None)
    add("xref-stream-bomb", raw=xref_stream(document([plain]), body=b"\0" * (CAP + 1), compressed=True), error="pdf_stream_limit")
    add("xref-width-bomb", raw=xref_stream(document([plain]), extra=b"/W [9223372036854775807 4 2]"), error="invalid_pdf")
    add("xref-count-bomb", raw=xref_stream(document([plain]), extra=b"/Index [0 9223372036854775807]"), error="invalid_pdf")
    add("xref-invalid-index-fallback", raw=xref_stream(document([plain]), extra=b"/Index /Invalid"), error="invalid_pdf")
    add("xref-odd-index", raw=xref_stream(document([plain]), extra=b"/Index [0 8 0]"), error="invalid_pdf")
    add("long-multibyte", document([b"BT /F1 12 Tf <" + b"e1" * 5000 + b"> Tj ET\n"], font=FONT.replace(b"StandardEncoding", b"WinAnsiEncoding"), compressed=True), ["á" * 5000 + "\n"])
    add("100-pages", document([plain] * 100), ["Hello PDF\n"] * 100)
    add("101-pages", document([plain] * 101), error="pdf_page_limit")
    add("zero-pages", document([]), error="pdf_page_limit")
    for size in [CAP - 1, CAP]:
        add(f"text-{size + 1}", document([b"BT /F1 12 Tf (" + b"a" * size + b") Tj ET\n"], compressed=True),
            texts=["a" * size + "\n"] if size == CAP - 1 else None,
            error="pdf_text_limit" if size == CAP else None)
    for size in [4 * CAP, 4 * CAP + 1]:
        content = b"%" + b"c" * (size - 2) + b"\n"
        add(f"page-stream-{size}", document([content], compressed=True),
            texts=[""] if size == 4 * CAP else None, error="pdf_stream_limit" if size > 4 * CAP else None)
    base = serialize(document([plain]))
    for size in [CAP, CAP + 1]:
        padding = size - len(base)
        source = serialize(document([plain]), padding=padding)
        padding -= len(source) - size  # startxref crece de tres a siete dígitos.
        source = serialize(document([plain]), padding=padding)
        assert len(source) == size
        add(f"source-{size}", raw=source, texts=["Hello PDF\n"] if size == CAP else None,
            error="invalid_pdf" if size > CAP else None)
    add("encrypted-empty", raw=encrypted(b""), error="pdf_encrypted")
    add("encrypted-password", raw=encrypted(b"synthetic-user"), error="pdf_encrypted")
    for name, bad in [
        ("type0", FONT.replace(b"/Type1", b"/Type0")),
        ("truetype", FONT.replace(b"/Type1", b"/TrueType")),
        ("type3", FONT.replace(b"/Type1", b"/Type3")),
        ("symbol", FONT.replace(b"/Helvetica", b"/Symbol")),
        ("embedded", FONT.replace(b" >>", b" /FontDescriptor << /FontFile 4 0 R >> >>")),
        ("tounicode", FONT.replace(b" >>", b" /ToUnicode 4 0 R >>")),
        ("differences", FONT.replace(b"/StandardEncoding", b"<< /Type /Encoding /Differences [65 /B] >>")),
        ("macroman", FONT.replace(b"/StandardEncoding", b"/MacRomanEncoding")),
        ("unknown-encoding", FONT.replace(b"/StandardEncoding", b"/UnknownEncoding")),
    ]:
        add(name, document([plain], font=bad), error="pdf_unsupported_feature")
    for name, content, error in [
        ("missing-glyph", b"BT /F1 12 Tf <01> Tj ET\n", "pdf_decode_error"),
        ("nul", b"BT /F1 12 Tf <00> Tj ET\n", "pdf_decode_error"),
        ("unknown-operator", plain + b"Mystery\n", "pdf_unsupported_feature"),
        ("actualtext", b"/Span << /ActualText (Hidden) >> BDC " + plain + b"EMC\n", "pdf_unsupported_feature"),
        ("content-duplicate-property", b"/Span << /MCID 1 /MCID 0 >> BDC " + plain + b"EMC\n", "pdf_decode_error"),
        ("content-duplicate-escaped-property", b"/Span << /MCID 1 /#4dCID 0 >> BDC " + plain + b"EMC\n", "pdf_decode_error"),
        ("nested-bt", b"BT BT ET ET\n", "invalid_pdf"),
        ("missing-et", b"BT /F1 12 Tf (A) Tj\n", "invalid_pdf"),
        ("text-outside-bt", b"(A) Tj\n", "invalid_pdf"),
        ("no-font", b"BT (A) Tj ET\n", "invalid_pdf"),
        ("bad-tj", b"BT /F1 12 Tf 12 Tj ET\n", "invalid_pdf"),
        ("bad-tj-array", b"BT /F1 12 Tf [(A) true] TJ ET\n", "invalid_pdf"),
        ("bad-tf", b"BT /F1 (bad) Tf ET\n", "invalid_pdf"),
        ("trailing-content", plain + b"[(\n", "pdf_decode_error"),
        ("stack-underflow", b"Q\n", "invalid_pdf"),
        ("marked-underflow", b"EMC\n", "invalid_pdf"),
        ("deep-graphics", b"q\n" * 65 + b"Q\n" * 65, "pdf_unsupported_feature"),
        ("inline-image", b"BI /W 1 /H 1 /BPC 8 /CS /G ID \xff EI\n", "pdf_unsupported_feature"),
    ]:
        add(name, document([content]), error=error)
    add("form-xobject", [obj.replace(b"/Subtype /Image", b"/Subtype /Form") for obj in document([b"/Im Do\n"])], error="pdf_unsupported_feature")
    add("annotation", document([plain], page_extra=b"/Annots [<< /Subtype /Text /Contents (Hidden) >>]"), error="pdf_unsupported_feature")
    add("acroform", document([plain], root_extra=b"/AcroForm << /Fields [] >>"), error="pdf_unsupported_feature")
    add("link-appearance", document([plain], page_extra=b"/Annots [<< /Subtype /Link /AP << >> >>]"), error="pdf_unsupported_feature")
    add("link-text", document([plain], page_extra=b"/Annots [<< /Subtype /Link /Contents (Hidden) >>]"), error="pdf_unsupported_feature")
    add("appearance-state", document([plain], resources=b"<< /ExtGState << /G << /Font [3 0 R 12] >> >> >>" )[:-1] + [stream(b"/G gs\n")], error="pdf_unsupported_feature")
    add("property-substitution", document([b"/Span /P BDC " + plain + b"EMC\n"], resources=RESOURCES[:-2] + b" /Properties << /P << /ActualText (Hidden) >> >> >>"), error="pdf_unsupported_feature")
    # Contents array se concatena antes de parsear: token y BT/ET cruzan streams.
    split = document([plain])
    split[4] = split[4].replace(b"/Contents 6 0 R", b"/Contents [6 0 R 7 0 R]")
    split[-1] = stream(b"BT /F1 12 Tf (Hel")
    split.append(stream(b"lo PDF) Tj ET\n"))
    add("split-stream-token", split, ["Hello PDF\n"])
    no_contents = document([b""])
    no_contents[4] = no_contents[4].replace(b"/Contents 6 0 R", b"")
    add("no-contents", no_contents, [""])
    nested = document([plain])
    nested[1] = b"<< /Type /Pages /Count 1 /Kids [7 0 R] /Resources " + RESOURCES + b" >>"
    nested[4] = nested[4].replace(b"/Parent 2 0 R", b"/Parent 7 0 R")
    nested.append(b"<< /Type /Pages /Parent 2 0 R /Count 1 /Kids [5 0 R] >>")
    add("nested-inheritance", nested, ["Hello PDF\n"])
    leaf_resources = document([plain], page_extra=b"/Resources << /Font << /F1 3 0 R >> >>", resources=b"<< >>")
    add("leaf-overrides-resources", leaf_resources, ["Hello PDF\n"])
    for name, objects in [
        ("leaf-resources-not-merged", document([plain], page_extra=b"/Resources << >>")),
        ("resources-wrong-type", document([plain], resources=b"[]")),
        ("font-resource-missing", document([plain], resources=b"<< /Font << >> >>")),
        ("font-resource-wrong-type", document([plain], resources=b"<< /Font [] >>")),
        ("font-generation-mismatch", document([plain], resources=RESOURCES.replace(b"3 0 R", b"3 1 R"))),
        ("contents-reference-cycle", document([plain])),
        ("resource-reference-cycle", document([plain], resources=b"7 0 R")),
        ("missing-page-parent", document([plain])),
        ("page-has-kids", document([plain], page_extra=b"/Kids []")),
        ("root-parent-invalid", document([plain])),
        ("kids-not-reference", document([plain])),
        ("negative-page-count", document([plain])),
    ]:
        if name == "contents-reference-cycle":
            objects[-1] = b"6 0 R"
        elif name == "resource-reference-cycle":
            objects.append(b"7 0 R")
        elif name == "missing-page-parent":
            objects[4] = objects[4].replace(b"/Parent 2 0 R", b"")
        elif name == "root-parent-invalid":
            objects[1] = objects[1].replace(b"/Type /Pages", b"/Type /Pages /Parent 1 0 R")
        elif name == "kids-not-reference":
            objects[1] = objects[1].replace(b"[5 0 R]", b"[<< /Type /Page >>]")
        elif name == "negative-page-count":
            objects[1] = objects[1].replace(b"/Count 1", b"/Count -1")
        add(name, objects, error="invalid_pdf")
    deep = document([plain])
    deep[0] = b"<< /Type /Catalog /Pages 2 0 R /Synthetic " + b"[" * 1000 + b"0" + b"]" * 1000 + b" >>"
    add("deep-object-array", deep, error="invalid_pdf")
    deep_content = b"[" * 1000 + b"0" + b"]" * 1000 + b" TJ\n"
    add("deep-content-array", document([deep_content]), error="pdf_decode_error")
    active = b"/OpenAction << /S /JavaScript /JS (throw Synthetic) >>"
    add("active-data", document([plain], root_extra=active, page_extra=b"/Annots [<< /Subtype /Link /A << /S /URI /URI (https://invalid.example/) >> >>]"), ["Hello PDF\n"])
    add("external-stream", document([plain])[:-1] + [stream(plain, extra=b" /F (https://invalid.example/)")], error="pdf_unsupported_feature")
    for name, objects in [("page-cycle", document([plain])), ("duplicate-page", document([plain])), ("bad-count", document([plain])), ("bad-parent", document([plain]))]:
        if name == "page-cycle":
            objects[1] = objects[1].replace(b"/Kids [5 0 R]", b"/Kids [2 0 R]")
        elif name == "duplicate-page":
            objects[1] = objects[1].replace(b"/Kids [5 0 R]", b"/Kids [5 0 R 5 0 R]").replace(b"/Count 1", b"/Count 2")
        elif name == "bad-count":
            objects[1] = objects[1].replace(b"/Count 1", b"/Count 2")
        else:
            objects[4] = objects[4].replace(b"/Parent 2 0 R", b"/Parent 1 0 R")
        add(name, objects, error="invalid_pdf")
    for name, source in [("truncated", base[:-8]), ("bad-signature", b"!" + base[1:]), ("preamble", b"junk" + base), ("version-2", base.replace(b"%PDF-1.7", b"%PDF-2.0")), ("xref-recovery", base.replace(b"startxref\n", b"startxref\n99%"))]:
        add(name, raw=source, error="invalid_pdf")
    replacement = stream(b"BT /F1 12 Tf (Revised) Tj ET\n")
    revised = revision(base, [(6, replacement)])
    add("incremental-newest-content", raw=revised, texts=["Revised\n"])
    add("incremental-three-revisions", raw=revision(revised, [(6, stream(plain))]), texts=["Hello PDF\n"])
    add("incremental-free-shadows-old", raw=revision(base, [(7, b"null")], size=8, free=[6]), error="invalid_pdf")
    add("incremental-unused-free", raw=revision(serialize(document([plain]) + [b"null"]), [(8, b"null")], size=9, free=[7]), texts=["Hello PDF\n"])
    broken_history = base.replace(b"endstream\nendobj", b"endstream\n      ")
    add("historical-replaced-missing-endobj", raw=revision(broken_history, [(6, replacement)]), error="invalid_pdf")
    add("historical-free-missing-endobj", raw=revision(serialize(document([plain]) + [b"null"]).replace(b"7 0 obj\nnull\nendobj", b"7 0 obj\nnull\n      "), [(8,b"null")], size=9, free=[7]), error="invalid_pdf")
    add("historical-eof-wrong-offset", raw=revision(base.replace(f"startxref\n{startxref(base)}\n".encode(), b"startxref\n0\n"), [(6,replacement)]), error="invalid_pdf")
    linear = linearized_framing()
    add("linearized-framing", raw=linear, texts=["Linear\n"])
    add("linearized-incremental", raw=revision(linear,[(6,replacement)],size=9),texts=["Revised\n"])
    add("linearized-missing-first-eof", raw=linear.replace(b"startxref\n0\n%%EOF", b"startxref\n0\n% EOF"), error="invalid_pdf")
    add("linearized-forward-prev-without-marker", raw=linear.replace(b"/Linearized", b"/OtherField"), error="invalid_pdf")
    add("hybrid-content-override",raw=hybrid_replacement(base,b"BT /F1 12 Tf (Hybrid) Tj ET\n"),texts=["Hybrid\n"])
    add("hybrid-three-revisions",raw=revision(hybrid_replacement(base,b"BT /F1 12 Tf (Hybrid) Tj ET\n"),[(6,replacement)],size=8),texts=["Revised\n"])
    commented = base.replace(b"1 0 obj\n",b"%comment 1 0 obj\n",1)
    # Volver a serializar posiciones exactas evita que el rechazo dependa de un
    # offset accidentalmente roto por el prefijo de comment.
    commented = commented.replace(b"0000000015 00000 n",b"0000000024 00000 n")
    for n in range(2,7):
        old=base.index(f"{n} 0 obj\n".encode());commented=commented.replace(f"{old:010} 00000 n".encode(),f"{old+9:010} 00000 n".encode())
    commented=commented.replace(f"startxref\n{startxref(base)}\n".encode(),f"startxref\n{startxref(base)+9}\n".encode())
    add("xref-object-in-comment",raw=commented,error="invalid_pdf")
    for key, value in [(b"Prev", b"/Invalid"), (b"Prev", b"null"), (b"Prev", b"1.0"), (b"Prev", b"-1"), (b"Prev", b"9999999999"),
                       (b"XRefStm", b"/Invalid"), (b"XRefStm", b"null"), (b"XRefStm", b"1.0"), (b"XRefStm", b"-1")]:
        add("trailer-" + key.decode() + "-" + value.decode().replace("/", "name-"),
            document([plain]), trailer=b"/" + key + b" " + value, error="invalid_pdf")
    add("prev-self-cycle", document([plain]), trailer=f"/Prev {startxref(base)}".encode(), error="invalid_pdf")
    placeholder = serialize(document([plain]), trailer=b"/Prev 0000000000")
    cyclic = revision(placeholder, [(6, replacement)])
    cyclic = cyclic.replace(b"/Prev 0000000000", f"/Prev {startxref(cyclic):010d}".encode(), 1)
    add("prev-two-node-cycle", raw=cyclic, error="invalid_pdf")
    add("duplicate-trailer-size", document([plain]), trailer=b"/Size 7", error="invalid_pdf")
    add("duplicate-trailer-escaped-size", document([plain]), trailer=b"/#53ize 7", error="invalid_pdf")
    add("duplicate-trailer-root", document([plain]), trailer=b"/Root 1 0 R", error="invalid_pdf")
    for count in [0, 6, 8, 9223372036854775807]:
        add(f"xref-subsection-count-{count}", raw=base.replace(b"xref\n0 7\n", f"xref\n0 {count}\n".encode(), 1), error="invalid_pdf")
    add("xref-subsection-overlap", raw=base.replace(b"trailer\n", b"1 1\n0000000015 00000 n \ntrailer\n", 1), error="invalid_pdf")
    add("xref-19-byte-row", raw=base.replace(b"0000000015 00000 n \n", b"0000000015 00000 n\n", 1), error="invalid_pdf")
    add("xref-wrong-row-generation", raw=base.replace(b"0000000015 00000 n \n", b"0000000015 99999 n \n", 1), error="invalid_pdf")
    add("startxref-inside-comment", raw=base.replace(b"startxref\n", b"%comment startxref\n"), error="invalid_pdf")
    # El texto de stream es opaco incluso si incluye tokens de objetos.
    overlap_objects = document([plain]) + [stream(b"8 0 obj\nnull\nendobj\n"), b"null"]
    overlap = serialize(overlap_objects)
    real_offset = overlap.rfind(b"8 0 obj\nnull")
    fake_offset = overlap.find(b"8 0 obj\nnull")
    overlap = overlap.replace(f"{real_offset:010d} 00000 n".encode(), f"{fake_offset:010d} 00000 n".encode())
    add("xref-object-inside-stream", raw=overlap, error="invalid_pdf")
    # Supplement hybrid sin efectos: el stream describe su propia entrada normal.
    supplement_offset = startxref(base)
    self_row = b"\1" + supplement_offset.to_bytes(4, "big") + b"\0\0"
    supplement = stream(self_row, extra=b" /Type /XRef /Size 8 /W [1 4 2] /Index [7 1]")
    add("hybrid-matching-supplement", document([plain]) + [supplement],
        trailer=f"/XRefStm {supplement_offset}".encode(), texts=["Hello PDF\n"])
    add("hybrid-supplement-with-prev", document([plain]) + [stream(self_row, extra=b" /Type /XRef /Size 8 /W [1 4 2] /Index [7 1] /Prev 0")],
        trailer=f"/XRefStm {supplement_offset}".encode(), error="invalid_pdf")
    add("hybrid-supplement-not-stream", document([plain]), trailer=b"/XRefStm 186", error="invalid_pdf")
    add("hybrid-cycle", document([plain]), trailer=f"/XRefStm {startxref(base)}".encode(), error="invalid_pdf")
    add("hybrid-supplement-size-mismatch", document([plain]) + [stream(self_row, extra=b" /Type /XRef /Size 9 /W [1 4 2] /Index [7 1]")],
        trailer=f"/XRefStm {supplement_offset}".encode(), error="invalid_pdf")
    # Un suplemento que contradice la tabla no puede ignorarse, aunque no cambie
    # el número de páginas. La biblioteca actual no respeta esa precedencia.
    add("hybrid-supplement-free-precedence", document([plain]) + [stream(b"\0\0\0\0\0\0\1", extra=b" /Type /XRef /Size 8 /W [1 4 2] /Index [6 1]")],
        trailer=f"/XRefStm {supplement_offset}".encode(), error="invalid_pdf")
    corrupt = document([plain], compressed=True)
    payload = zlib.compress(plain, 9)
    corrupt[-1] = corrupt[-1].replace(payload, payload[:-1] + bytes([payload[-1] ^ 1]))
    add("flate-bad-checksum", corrupt, error="pdf_decode_error")
    corrupt = document([plain])
    corrupt[-1] = corrupt[-1].replace(f"/Length {len(plain)}".encode(), b"/Length 1")
    add("wrong-stream-length", corrupt, error="invalid_pdf")
    add("missing-endobj", raw=base.replace(b"\nendobj\n", b"\n      \n", 1), error="invalid_pdf")
    for name, key in [("duplicate-basefont", b"BaseFont"), ("duplicate-escaped-key", b"Base#46ont")]:
        duplicate = document([plain])
        duplicate[2] = FONT.replace(b"/BaseFont /Helvetica", b"/" + key + b" /Unsupported /BaseFont /Helvetica")
        add(name, duplicate, error="invalid_pdf")
    compressed_duplicate = document([plain]) + [stream(b"8 0 << /A 1 /A 2 >>", True, b" /Type /ObjStm /N 1 /First 4")]
    add("compressed-duplicate-dictionary", compressed_duplicate, error="invalid_pdf")
    # Los mismos filtros/predictors sobre streams de carga, no sólo Contents.
    transforms = [
        ("asciihex", lambda b: b.hex().encode()+b">", b" /Filter /ASCIIHexDecode"),
        ("ascii85", lambda b: base64.a85encode(b)+b"~>", b" /Filter /ASCII85Decode"),
        ("runlength", lambda b: b"".join(bytes([len(b[i:i+128])-1])+b[i:i+128] for i in range(0,len(b),128))+b"\x80", b" /Filter /RunLengthDecode"),
        ("lzw-0", lambda b: lzw_codes([256,*b,257],0), b" /Filter /LZWDecode /DecodeParms << /EarlyChange 0 >>"),
        ("lzw-1", lambda b: lzw_codes([256,*b,257],1), b" /Filter /LZWDecode /DecodeParms << /EarlyChange 1 >>"),
        ("tiff", lambda b: zlib.compress(b,9), b" /Filter /FlateDecode /DecodeParms << /Predictor 2 /Columns 1 >>"),
        ("png", lambda b: zlib.compress(png_encode(b,1),9), b" /Filter /FlateDecode /DecodeParms << /Predictor 15 /Columns 1 >>"),
    ]
    for name, transform, extra in transforms:
        add("load-xref-"+name, raw=xref_stream(document([plain]),transform=transform,extra=extra), texts=["Hello PDF\n"])
        objs = document([plain],resources=RESOURCES.replace(b"3 0 R",b"8 0 R"))
        member = b"8 0 " + FONT
        objs += [stream(transform(member),extra=extra+b" /Type /ObjStm /N 1 /First 4"),None]
        add("load-object-"+name,raw=xref_stream(objs,compressed_entries={8:(7,0)}),texts=["Hello PDF\n"])
    return result


def manifest():
    entries = []
    for case in cases():
        entry = {"name": case["name"], "source_bytes": len(case["source"]),
                 "source_sha256": hashlib.sha256(case["source"]).hexdigest()}
        if case["error"]:
            entry["error"] = case["error"]
        else:
            entry["pages"] = [{"page_number": index, "state": "text" if text else "no_text",
                               "text_bytes": len(text.encode()), "text_sha256": hashlib.sha256(text.encode()).hexdigest()}
                              for index, text in enumerate(case["texts"], 1)]
        entries.append(entry)
    return {"seed": SEED, "license": "MIT", "generator": "generate.py (sin librería PDF)",
            "zlib_runtime": zlib.ZLIB_RUNTIME_VERSION, "cases": entries}


if __name__ == "__main__":
    # Sólo salida textual: el operador elige el archivo de evidencia.
    print(json.dumps(manifest(), indent=2, ensure_ascii=False))
