import { zipSync, strToU8 } from 'fflate';

const XML_HEADER = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>';

const escapeXml = (value) => String(value ?? '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&apos;');

const sheetName = (name, index) => {
  const cleaned = String(name || `Sheet ${index + 1}`)
    .replace(/[\[\]:*?/\\]/g, ' ')
    .trim();

  return (cleaned || `Sheet ${index + 1}`).slice(0, 31);
};

const columnName = (index) => {
  let name = '';
  let n = index + 1;

  while (n > 0) {
    const rem = (n - 1) % 26;
    name = String.fromCharCode(65 + rem) + name;
    n = Math.floor((n - 1) / 26);
  }

  return name;
};

const cellXml = (value, rowIndex, columnIndex) => {
  const ref = `${columnName(columnIndex)}${rowIndex + 1}`;

  if (typeof value === 'number' && Number.isFinite(value)) {
    return `<c r="${ref}"><v>${value}</v></c>`;
  }

  if (typeof value === 'boolean') {
    return `<c r="${ref}" t="b"><v>${value ? 1 : 0}</v></c>`;
  }

  return `<c r="${ref}" t="inlineStr"><is><t>${escapeXml(value)}</t></is></c>`;
};

const worksheetXml = (rows) => {
  const normalizedRows = rows.map((row) => Array.isArray(row) ? row : [row]);
  const maxColumns = normalizedRows.reduce((max, row) => Math.max(max, row.length), 0);
  const dimension = normalizedRows.length && maxColumns
    ? `A1:${columnName(maxColumns - 1)}${normalizedRows.length}`
    : 'A1';

  const rowsXml = normalizedRows.map((row, rowIndex) => (
    `<row r="${rowIndex + 1}">${row.map((value, columnIndex) => cellXml(value, rowIndex, columnIndex)).join('')}</row>`
  )).join('');

  return `${XML_HEADER}<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><dimension ref="${dimension}"/><sheetData>${rowsXml}</sheetData></worksheet>`;
};

const workbookXml = (sheets) => `${XML_HEADER}<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>${sheets.map((sheet, index) => `<sheet name="${escapeXml(sheet.name)}" sheetId="${index + 1}" r:id="rId${index + 1}"/>`).join('')}</sheets></workbook>`;

const workbookRelsXml = (sheets) => `${XML_HEADER}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">${sheets.map((_, index) => `<Relationship Id="rId${index + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet${index + 1}.xml"/>`).join('')}</Relationships>`;

const rootRelsXml = `${XML_HEADER}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`;

const contentTypesXml = (sheets) => `${XML_HEADER}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>${sheets.map((_, index) => `<Override PartName="/xl/worksheets/sheet${index + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>`).join('')}</Types>`;

const downloadBlob = (blob, filename) => {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
};

export const createWorkbookBytes = (sheets) => {
  const normalizedSheets = sheets.map((sheet, index) => ({
    name: sheetName(sheet.name, index),
    rows: Array.isArray(sheet.rows) ? sheet.rows : [],
  }));

  const files = {
    '[Content_Types].xml': strToU8(contentTypesXml(normalizedSheets)),
    '_rels/.rels': strToU8(rootRelsXml),
    'xl/workbook.xml': strToU8(workbookXml(normalizedSheets)),
    'xl/_rels/workbook.xml.rels': strToU8(workbookRelsXml(normalizedSheets)),
  };

  normalizedSheets.forEach((sheet, index) => {
    files[`xl/worksheets/sheet${index + 1}.xml`] = strToU8(worksheetXml(sheet.rows));
  });

  return zipSync(files, { level: 6 });
};

export const exportWorkbook = (sheets, filename) => {
  const blob = new Blob([createWorkbookBytes(sheets)], {
    type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  });

  downloadBlob(blob, filename.endsWith('.xlsx') ? filename : `${filename}.xlsx`);
};
