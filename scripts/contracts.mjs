import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { compile } from 'json-schema-to-typescript';

const schema = JSON.parse(execFileSync('cargo', ['run', '--quiet', '-p', 'noteka-core', '--example', 'schema'], { encoding: 'utf8' }));
const files = {
  'src/lib/contract.schema.json': JSON.stringify(schema, null, 2) + '\n',
  'src/lib/contracts.generated.ts': await compile(schema, 'Contract', { bannerComment: '/* Generado desde noteka-core. No editar; npm run contracts:generate. */', additionalProperties: false }),
};
for (const [path, content] of Object.entries(files)) {
  if (process.argv.includes('--check')) {
    if (readFileSync(path, 'utf8') !== content) throw new Error(`${path} desactualizado. Ejecutá npm run contracts:generate.`);
  } else writeFileSync(path, content);
}
