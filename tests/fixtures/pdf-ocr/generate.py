#!/usr/bin/env python3
"""PDF propios MIT: escaneo sin capa textual y variantes. Golden previo al motor."""
import hashlib, importlib.util, json, os, sys
from pathlib import Path
root=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('pdf',root/'tests/fixtures/pdf/generate.py');pdf=importlib.util.module_from_spec(spec);spec.loader.exec_module(pdf)
out=Path(sys.argv[1]).resolve(strict=True);base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
assert out.is_relative_to(base) and not (out/'manifest.json').exists()
ground=json.loads((out/'image.json').read_text());gray=(out/'clean.gray').read_bytes();assert len(gray)==1200*320

def scanned(count=1,rotation=0,box=(0,0,600,160),blank=(),noise=False,mixed=False,upright_rotation=False):
    objects=[b'<< /Type /Catalog /Pages 2 0 R >>',b'',pdf.FONT]
    if noise:
        state=42;data=bytearray()
        for _ in range(1_000_000):
            state=(1664525*state+1013904223)&0xffffffff;data.append(state>>24)
        width=height=1000;data=bytes(data)
    else: width,height,data=1200,320,gray
    if upright_rotation:
        for _ in range(((360-rotation)%360)//90):
            data=bytes(data[(height-1-x)*width+y] for y in range(width) for x in range(height));width,height=height,width
    objects.append(pdf.stream(data,compressed=True,extra=f' /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceGray /BitsPerComponent 8'.encode()))
    kids=[]
    for p in range(1,count+1):
        n=len(objects)+1;kids.append(f'{n} 0 R'.encode());x,y,x2,y2=box
        content=b'' if p in blank else f'q {x2-x} 0 0 {y2-y} {x} {y} cm /Im Do Q\n'.encode()
        if mixed and p==2:content=b'BT /F1 18 Tf 10 60 Td (Mixed text layer 2026) Tj ET'
        objects += [f'<< /Type /Page /Parent 2 0 R /MediaBox [{x} {y} {x2} {y2}] /Rotate {rotation} /Contents {n+1} 0 R >>'.encode(),pdf.stream(content,compressed=True)]
    objects[1]=b'<< /Type /Pages /Count '+str(count).encode()+b' /Kids ['+b' '.join(kids)+b'] /Resources '+pdf.RESOURCES+b' >>'
    return pdf.serialize(objects)

cases=[]
def add(name,raw,**kw):
    assert 0<len(raw)<=1048576,(name,len(raw));path=out/(name+'.pdf');path.write_bytes(raw)
    cases.append(dict(name=path.name,sha256=hashlib.sha256(raw).hexdigest(),source_bytes=len(raw),**kw))
if '--ui' in sys.argv:
    add('regions',scanned(13,blank=(2,12)),pages=13,rotation=0,width=1200,height=320,quality=True,expected=[[] if p in (2,12) else ground['lines'] for p in range(1,14)])
    (out/'manifest.json').write_text(json.dumps(dict(license='MIT',expected_before_ocr=True,quality_profile='clean_printed_synthetic_subset',max_aggregate_cer=0.05,cases=cases),ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(cases=len(cases),root=str(out))));sys.exit(0)
add('scanned',scanned(),pages=1,rotation=0,width=1200,height=320,quality=True,expected=[ground['lines']])
for r in [90,180,270]:add(f'rotate-{r}',scanned(rotation=r),pages=1,rotation=r,width=320 if r in [90,270] else 1200,height=1200 if r in [90,270] else 320,quality=False)
for r in [90,180,270]:add(f'upright-rotate-{r}',scanned(rotation=r,box=(0,0,160,600) if r in [90,270] else (0,0,600,160),upright_rotation=True),pages=1,rotation=r,width=1200,height=320,quality=True,expected=[ground['lines']])
add('mixed',scanned(3,blank=(3,),mixed=True),pages=3,rotation=0,width=1200,height=320,quality=True,expected=[ground['lines'],['Mixed text layer 2026'],[]])
add('blank',scanned(blank=(1,)),pages=1,rotation=0,width=1200,height=320,quality=True,expected=[[]])
add('offset-box',scanned(box=(10,-20,610,140)),pages=1,rotation=0,width=1200,height=320,quality=True,expected=[ground['lines']])
add('twenty-pages',scanned(20,box=(0,0,100,100),blank=tuple(range(1,21))),pages=20,rotation=0,width=200,height=200,quality=False)
add('twenty-one-pages',scanned(21,box=(0,0,100,100)),error='pdf_ocr_page_limit')
add('document-pixels-exact',scanned(4,box=(0,0,1000,1000),blank=(1,2,3,4)),pages=4,rotation=0,width=2000,height=2000,quality=False)
add('document-pixels',scanned(5,box=(0,0,1000,1000)),error='pdf_ocr_pixel_limit')
add('geometry',scanned(box=(0,0,1000001,100)),error='pdf_ocr_geometry_limit')
add('raster-bytes',scanned(noise=True,box=(0,0,1500,1500)),error='pdf_ocr_raster_limit')
for password in [b'',b'synthetic-user']:add('encrypted-'+('empty' if not password else 'password'),pdf.encrypted(password),error='pdf_encrypted')
exact=scanned();add('source-exact',exact+b' '*(1048576-len(exact)),pages=1,rotation=0,width=1200,height=320,quality=True,expected=[ground['lines']])
add('bad-framing',b'not a pdf',error='pdf_invalid_framing')
add('invalid-document',b'%PDF-1.7\ninvalid\n%%EOF',error='pdf_invalid_document')
(out/'manifest.json').write_text(json.dumps(dict(license='MIT',expected_before_ocr=True,quality_profile='clean_printed_synthetic_subset',max_aggregate_cer=0.05,cases=cases),ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),root=str(out))))
