"""Build the bounded SLICE demo, native references and inspectable source custody."""
import argparse, hashlib, html, json, os, shutil, subprocess, zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def capture(*args):return subprocess.check_output(args,cwd=ROOT).decode('utf-8').strip()
def run(*args):subprocess.run(args,cwd=ROOT,check=True)
def pin(path):return {'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('dist/SLICE'));a=p.parse_args()
 output=(ROOT/a.output).resolve();dist=(ROOT/'dist').resolve()
 if output==dist or not output.is_relative_to(dist):raise ValueError('Output must be a child of dist')
 if capture('wasm-bindgen','--version')!='wasm-bindgen 0.2.127':raise ValueError('Use wasm-bindgen 0.2.127')
 run('cargo','build','--locked','--release','--target','wasm32-unknown-unknown','-p','slice-web','--lib','--features','wasm')
 run('cargo','build','--locked','-p','slice-web','--bin','slice-playground')
 metadata=json.loads(capture('cargo','metadata','--locked','--format-version','1','--filter-platform','wasm32-unknown-unknown','--features','slice-web/wasm'))
 if output.exists():
  def linked(path):return path.is_symlink() or bool(getattr(path.lstat(),'st_file_attributes',0)&1024)
  if linked(output) or any(linked(p) for p in output.rglob('*')):raise ValueError('Linked output refused')
  shutil.rmtree(output)
 output.mkdir(parents=True)
 target=Path(metadata['target_directory']);run('wasm-bindgen',str(target/'wasm32-unknown-unknown/release/slice_web.wasm'),'--target','web','--out-dir',str(output/'pkg'))
 sample=json.loads((ROOT/'web/example-request.json').read_text(encoding='utf-8'))
 page=(ROOT/'web/index.html').read_text(encoding='utf-8')
 for key,value in [('RECORDS',sample['records_json']),('CATALOG',sample['catalog_json']),('EXPRESSION',sample['expression'])]:page=page.replace('{{'+key+'}}',html.escape(value))
 (output/'index.html').write_text(page,encoding='utf-8')
 for name in ('app.js','worker.js','style.css'):shutil.copyfile(ROOT/'web'/name,output/name)
 for name in ('LICENSE','BUILD.md'):shutil.copyfile(ROOT/name,output/name)
 shutil.copytree(ROOT/'tools/licenses/rust',output/'rust')
 # Include notices for the portable adapter's actual resolved dependency closure,
 # including build-time proc macros, rather than the native SQLite workspace.
 packages={p['id']:p for p in metadata['packages']};nodes={n['id']:n for n in metadata['resolve']['nodes']}
 first=next(p['id'] for p in metadata['packages'] if p['name']=='slice-web');pending=[first];visited=set();notices=[];dependencies=[]
 while pending:
  current=pending.pop()
  if current in visited:continue
  visited.add(current);pending.extend(nodes[current]['dependencies']);pkg=packages[current]
  if pkg['source'] is None:continue
  base=Path(pkg['manifest_path']).parent
  files=sorted({f for pattern in ('LICENSE*','COPYRIGHT*','NOTICE*') for f in base.glob(pattern) if f.is_file()})
  if not files:raise ValueError('No license notice found for '+pkg['name'])
  dependencies.append({'name':pkg['name'],'version':pkg['version'],'license':pkg['license']})
  notices.append('\n\n===== '+pkg['name']+' '+pkg['version']+' ('+str(pkg['license'])+') =====\n')
  for f in files:notices.append('\n--- '+f.name+' ---\n'+f.read_text(encoding='utf-8'))
 notice='SLICE portable adapter dependency closure, including build-time proc macros.\nNative SQLite/CLI-only dependencies are excluded. Original licenses follow.\nRust standard-library notices: rust/COPYRIGHT-library.html and rust/licenses/.\n'+''.join(notices)
 (output/'THIRD-PARTY-NOTICES.txt').write_text(notice,encoding='utf-8')
 dirty=bool(capture('git','status','--porcelain','--untracked-files=all'));commit=capture('git','rev-parse','HEAD')
 tracked=capture('git','ls-files','-z').split('\0')
 if dirty:
  with zipfile.ZipFile(output/'source.zip','w',compression=zipfile.ZIP_DEFLATED) as archive:
   for name in tracked:
    path=ROOT/name
    if path.is_file():
     if not path.resolve().is_relative_to(ROOT):raise ValueError('External source path refused')
     archive.write(path,name)
 else:run('git','archive','--format=zip','--output='+str(output/'source.zip'),'HEAD')
 with zipfile.ZipFile(output/'source.zip') as archive:
  for name in ('crates/slice-web/src/lib.rs','crates/slice-core/src/lib.rs','Cargo.lock','BUILD.md'):
   expected=(ROOT/name).read_bytes() if dirty else subprocess.check_output(['git','show',commit+':'+name],cwd=ROOT)
   if archive.read(name)!=expected:raise ValueError('Source ZIP differs from build source: '+name)
 for name in ('crates/slice-web/src/lib.rs','crates/slice-core/src/lib.rs','Cargo.lock'):
  dest=output/'source'/name;dest.parent.mkdir(parents=True,exist_ok=True)
  dest.write_bytes((ROOT/name).read_bytes() if dirty else subprocess.check_output(['git','show',commit+':'+name],cwd=ROOT))
 provenance={'schema':'slice.pages-build.v1','commit':commit,'dirty':dirty,'source':'https://github.com/giodl73-repo/SLICE',
   'rust':capture('rustc','--version'),'bindings':capture('wasm-bindgen','--version'),'dependencies':sorted(dependencies,key=lambda p:p['name']),
   'fixture_rows':4,'delivery_budget_bytes':5_000_000}
 (output/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n',encoding='utf-8');(output/'.nojekyll').touch()
 cache=ROOT/'.cache';cache.mkdir(exist_ok=True);binary=target/'debug'/('slice-playground.exe' if os.name=='nt' else 'slice-playground')
 cases=[]
 def case(name,expression,records=None,catalog='default'):
  request={'records_json':sample['records_json'] if records is None else records,'expression':expression,'catalog_json':sample['catalog_json'] if catalog=='default' else catalog}
  path=cache/'request.json';path.write_text(json.dumps(request),encoding='utf-8')
  canonical=capture(str(binary),str(path));json.loads(canonical);cases.append({'name':name,'request':request,'canonical':canonical})
 case('default',sample['expression']);case('tags',"tags has 'public'");case('missing',"score is null")
 case('empty',"id eq 'absent'");case('untyped',"not (status eq 'held') and tags has any ['public']",catalog='')
 case('unknown-unicode',"title eq 'café' and missing eq 'x'")
 case('bad-selector',"score frobnicate 6");case('bad-records',"score gt 0",records='[broken')
 case('bad-catalog',"id eq 'guide'",catalog='{"fields":{"id":"invalid"}}')
 case('max-rows',"score ge 1999",records=json.dumps([{'score':i} for i in range(2000)]),catalog='{"fields":{"score":"number"}}')
 case('too-many-rows',"id is null",records=json.dumps([{}]*2001))
 case('quote-in-path',"x' eq 1",records=json.dumps([{"x'":1}]),catalog='')
 case('quote-depth-bound',"name eq 'x\\' and "+'('*17+'id is null'+')'*17,catalog='')
 (cache/'native-references.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
 files={p.relative_to(output).as_posix():pin(p) for p in sorted(output.rglob('*')) if p.is_file()}
 size=sum(p['bytes'] for p in files.values())
 if size>=5_000_000:raise ValueError('Static demo exceeds the 5 MB delivery budget')
 (cache/'site-inventory.json').write_text(json.dumps({'commit':commit,'dirty':dirty,'bytes':size,'files':files},indent=2)+'\n')
 print(json.dumps({'output':str(output),'bytes':size,'files':len(files),'native_reference_cases':len(cases),'dirty':dirty}))

if __name__=='__main__':main()
