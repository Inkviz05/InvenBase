import { describe, expect, it } from 'vitest';
import { strFromU8, unzipSync } from 'fflate';
import { createWorkbookBytes } from './xlsxExport';

describe('xlsxExport', () => {
  it('creates workbook package with escaped worksheet values', () => {
    const bytes = createWorkbookBytes([
      {
        name: 'Audit/Report:*?',
        rows: [
          ['Name', 'Count'],
          ['A&B <test>', 42],
        ],
      },
    ]);

    const files = unzipSync(bytes);

    expect(files['[Content_Types].xml']).toBeDefined();
    expect(files['_rels/.rels']).toBeDefined();
    expect(files['xl/workbook.xml']).toBeDefined();
    expect(files['xl/worksheets/sheet1.xml']).toBeDefined();

    const workbook = strFromU8(files['xl/workbook.xml']);
    const sheet = strFromU8(files['xl/worksheets/sheet1.xml']);

    expect(workbook).toContain('Audit Report');
    expect(sheet).toContain('A&amp;B &lt;test&gt;');
    expect(sheet).toContain('<v>42</v>');
  });
});
