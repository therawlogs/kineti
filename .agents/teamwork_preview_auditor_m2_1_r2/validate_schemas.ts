import Ajv2020 from "ajv/dist/2020";
import fs from "fs";

const content = fs.readFileSync("docs/HARNESS_STRATEGY_BLUEPRINT.md", "utf8");

// Extract Appendix B schema
const appBMatch = content.match(/### 8\.2 Appendix B:[\s\S]*?```json\s*([\s\S]*?)\s*```/);
if (!appBMatch) {
  console.error("FAIL: Could not extract Appendix B schema");
  process.exit(1);
}
const appBSchema = JSON.parse(appBMatch[1]);

// Extract Appendix C schema
const appCMatch = content.match(/### 8\.3 Appendix C:[\s\S]*?```json\s*([\s\S]*?)\s*```/);
if (!appCMatch) {
  console.error("FAIL: Could not extract Appendix C schema");
  process.exit(1);
}
const appCSchema = JSON.parse(appCMatch[1]);

// Extract OVT Instance in Section 3.5
const ovtInstanceMatch = content.match(/```json\s*(\{\s*"@context":\s*\[[\s\S]*?"proof":\s*\[[\s\S]*?\}\s*\]\s*\})\s*```/);
if (!ovtInstanceMatch) {
  console.error("FAIL: Could not extract Section 3.5 OVT instance");
  process.exit(1);
}
const ovtInstance = JSON.parse(ovtInstanceMatch[1]);

const ajv = new Ajv2020({ allErrors: true, strict: false });

try {
  const validateB = ajv.compile(appBSchema);
  console.log("PASS: Appendix B schema compiled successfully with Ajv2020");
} catch (e) {
  console.error("FAIL: Appendix B schema compilation failed:", e);
  process.exit(1);
}

try {
  const validateC = ajv.compile(appCSchema);
  console.log("PASS: Appendix C schema compiled successfully with Ajv2020");
  
  const valid = validateC(ovtInstance);
  if (valid) {
    console.log("PASS: Section 3.5 OVT instance validates perfectly against Appendix C schema!");
  } else {
    console.error("FAIL: Section 3.5 OVT instance failed validation:", validateC.errors);
    process.exit(1);
  }
} catch (e) {
  console.error("FAIL: Appendix C schema compilation failed:", e);
  process.exit(1);
}
