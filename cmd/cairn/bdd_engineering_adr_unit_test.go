package main

import (
	"testing"

	"github.com/jeduden/cairn/internal/adr"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// eng18Row is an SRS stand-in holding just the ENG-18 row the
// helpers read the allow-list and target from.
const eng18Row = `# 10

| ID     | Pri | Requirement                                                                     | Ver |
| ------ | --- | ------------------------------------------------------------------------------- | --- |
| ENG-18 | P0  | Licenses MUST be on an allow-list (MIT, BSD). The target is ≤ 2 direct dependencies. | I   |
`

func depADR(id, status, alternatives string, mods ...adr.Module) adr.ADR {
	return adr.ADR{
		Path: "docs/adr/" + id + "-x.md", ID: id, Title: "t", Status: status, Summary: "s",
		Sections: map[string]string{"Alternatives": alternatives}, Modules: mods,
	}
}

func mod(path, license string) adr.Module {
	return adr.Module{Path: path, Purpose: "p", License: license, Maintenance: "m"}
}

func TestReadADRsLoadsTheDirectory(t *testing.T) {
	e := checkout(t, map[string]string{"docs/adr/ADR-01-a.md": "---\nid: ADR-01\n---\n"})

	require.NoError(t, e.readADRs("docs/adr"))
	assert.Equal(t, "docs/adr", e.adrDir)
	require.Len(t, e.adrs, 1)
	assert.Equal(t, "ADR-01", e.adrs[0].ID)

	e = checkout(t, map[string]string{"docs/adr/bad.md": "no front matter"})
	assert.ErrorContains(t, e.readADRs("docs/adr"), "no front matter")
}

func TestDepsNamedOnceAndModulesAreDeps(t *testing.T) {
	e := &engineering{deps: []string{"a", "b", "c"}, adrs: []adr.ADR{
		depADR("ADR-01", adr.Accepted, "x", mod("a", "MIT"), mod("stale", "MIT")),
		depADR("ADR-02", adr.Accepted, "x", mod("b", "MIT")),
		depADR("ADR-03", adr.Accepted, "x", mod("b", "MIT")),
		depADR("ADR-04", adr.Superseded, "x", mod("c", "MIT")),
	}}

	err := e.depsNamedOnce()
	assert.ErrorContains(t, err, "b is named by 2 accepted ADRs [ADR-02 ADR-03]")
	assert.ErrorContains(t, err, "c is named by 0 accepted ADRs")
	assert.NotContains(t, err.Error(), "a is named")
	assert.EqualError(t, e.modulesAreDeps(), "ADR-01 names stale, which is not a direct dependency")

	e.adrs = e.adrs[1:2]
	e.deps = []string{"b"}
	assert.NoError(t, e.depsNamedOnce())
	assert.NoError(t, e.modulesAreDeps())
}

func TestDepsJustified(t *testing.T) {
	e := &engineering{adrs: []adr.ADR{
		depADR("ADR-01", adr.Accepted, "", adr.Module{Path: "a", License: "MIT", Maintenance: "m"}),
		depADR("ADR-02", adr.Accepted, ""),
		depADR("ADR-03", adr.Proposed, "", adr.Module{Path: "b"}),
	}}

	err := e.depsJustified()
	assert.ErrorContains(t, err, "ADR-01 weighs no alternatives")
	assert.ErrorContains(t, err, `ADR-01 leaves "Purpose" empty for a`)
	assert.NotContains(t, err.Error(), "ADR-02")
	assert.NotContains(t, err.Error(), "ADR-03")

	e.adrs = []adr.ADR{depADR("ADR-04", adr.Accepted, "x", mod("a", "MIT"))}
	assert.NoError(t, e.depsJustified())
}

func TestLicensesAndTargetComeFromTheRequirement(t *testing.T) {
	e := checkout(t, map[string]string{"docs/srs/10.md": eng18Row})
	e.deps = []string{"a", "b"}
	e.adrs = []adr.ADR{depADR("ADR-01", adr.Accepted, "x", mod("a", "MIT"), mod("b", "BSD-3-Clause"))}

	assert.NoError(t, e.licensesAllowed())
	assert.NoError(t, e.withinTarget())

	e.adrs = []adr.ADR{depADR("ADR-01", adr.Accepted, "x", mod("a", "GPL-3.0"), mod("b", "BSDish"))}
	err := e.licensesAllowed()
	assert.ErrorContains(t, err, `a is licensed "GPL-3.0"`)
	assert.ErrorContains(t, err, `b is licensed "BSDish"`)

	e.deps = []string{"a", "b", "c"}
	assert.ErrorContains(t, e.withinTarget(), "3 direct dependencies, want at most 2")
}

func TestRequirementReadsReportWhatIsMissing(t *testing.T) {
	noRow := checkout(t, map[string]string{"docs/srs/10.md": "# 10\n"})
	assert.ErrorContains(t, noRow.licensesAllowed(), "no requirement ENG-18 in the SRS")

	silent := checkout(t, map[string]string{"docs/srs/10.md": `# 10

| ID     | Pri | Requirement       | Ver |
| ------ | --- | ----------------- | --- |
| ENG-18 | P0  | Nothing said here | I   |
`})
	assert.ErrorContains(t, silent.licensesAllowed(), "ENG-18 does not state")
	assert.ErrorContains(t, silent.withinTarget(), "ENG-18 does not state")

	broken := checkout(t, map[string]string{"docs/srs/10.md": "| ID | Requirement |\n| - | - |\n| bad | x |\n"})
	assert.Error(t, broken.withinTarget())
}

func TestListsDependencyADRs(t *testing.T) {
	e := checkout(t, map[string]string{"DEPENDENCIES.md": "- [ADR-01](docs/adr/ADR-01-x.md)\n"})
	e.adrDir = "docs/adr"
	e.adrs = []adr.ADR{
		depADR("ADR-01", adr.Accepted, "x", mod("a", "MIT")),
		depADR("ADR-02", adr.Accepted, "x"),
	}

	assert.NoError(t, e.listsDependencyADRs("DEPENDENCIES.md"))

	e.adrs = append(e.adrs, depADR("ADR-03", adr.Superseded, "x", mod("b", "MIT")))
	assert.EqualError(t, e.listsDependencyADRs("DEPENDENCIES.md"), "DEPENDENCIES.md does not link docs/adr/ADR-03-x.md")
	assert.ErrorContains(t, e.listsDependencyADRs("missing.md"), "read missing.md")
}

func TestADRIdentityChecks(t *testing.T) {
	good := depADR("ADR-01", adr.Accepted, "x")
	e := &engineering{adrs: []adr.ADR{good}}
	assert.NoError(t, e.adrsComplete())
	assert.NoError(t, e.adrsNamedForID())
	assert.NoError(t, e.adrStatusesValid())
	assert.NoError(t, e.adrIDsUnique())
	assert.NoError(t, e.supersededNamesSuccessor())

	bad := adr.ADR{Path: "docs/adr/other.md", ID: "7", Status: "draft"}
	gone := depADR("ADR-02", adr.Superseded, "x")
	gone.SupersededBy = "ADR-99"
	e.adrs = []adr.ADR{good, good, bad, gone}

	err := e.adrsComplete()
	assert.ErrorContains(t, err, `docs/adr/other.md: id "7" is not ADR-<digits>`)
	assert.ErrorContains(t, err, "docs/adr/other.md: no title")
	assert.ErrorContains(t, err, "docs/adr/other.md: no summary")
	assert.EqualError(t, e.adrsNamedForID(), "docs/adr/other.md is not named for its id 7")
	assert.EqualError(t, e.adrStatusesValid(), `7: status "draft" is not proposed, accepted or superseded`)
	assert.EqualError(t, e.adrIDsUnique(), "docs/adr/ADR-01-x.md: id ADR-01 already used by docs/adr/ADR-01-x.md")
	assert.EqualError(t, e.supersededNamesSuccessor(), `ADR-02 is superseded by "ADR-99", which is no other ADR`)
}

func TestLicenseFamilyAdmitsOnlyItsPermissiveVariants(t *testing.T) {
	allowed := []string{"Apache-2.0", "MIT", "BSD", "ISC"}

	assert.True(t, licenseAllowed("BSD-2-Clause", allowed))
	assert.True(t, licenseAllowed("BSD-3-Clause", allowed))
	assert.True(t, licenseAllowed("MIT", allowed))
	for _, l := range []string{"BSD", "BSD-4-Clause", "BSD-Protection", "MIT-advertising", "Apache-2.0-foo"} {
		assert.False(t, licenseAllowed(l, allowed), l)
	}
}

func TestSupersessionNamesAnotherRecordAndMarksTheOldOne(t *testing.T) {
	self := depADR("ADR-01", adr.Superseded, "x")
	self.SupersededBy = "ADR-01"
	half := depADR("ADR-02", adr.Accepted, "x")
	half.SupersededBy = "ADR-03"
	e := &engineering{adrs: []adr.ADR{self, half, depADR("ADR-03", adr.Accepted, "x")}}

	err := e.supersededNamesSuccessor()
	assert.ErrorContains(t, err, `ADR-01 is superseded by "ADR-01", which is no other ADR`)
	assert.ErrorContains(t, err, `ADR-02 names successor ADR-03 but its status is "accepted", not superseded`)
}
