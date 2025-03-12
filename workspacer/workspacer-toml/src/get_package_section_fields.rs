crate::ix!();

impl GetPackageAuthors for CargoToml {
    type Error = CargoTomlError;

    fn get_package_authors(&self) -> Result<Option<Vec<String>>, Self::Error> {
        debug!("Attempting to retrieve `authors` from CargoToml directly.");
        // Example logic: read `self.content["package"]["authors"]` if it exists
        // This depends on how your code organizes the TOML. We'll show a typical approach:
        if let Some(pkg) = self.content().as_table().and_then(|t| t.get("package")) {
            if let Some(pkg_table) = pkg.as_table() {
                // In TOML, authors could be an array of strings. 
                if let Some(auth_arr) = pkg_table.get("authors").and_then(|v| v.as_array()) {
                    let mut result = Vec::new();
                    for author_val in auth_arr {
                        if let Some(author_str) = author_val.as_str() {
                            result.push(author_str.to_string());
                        }
                    }
                    trace!("Found authors array in CargoToml: {:?}", result);
                    return Ok(Some(result));
                }
            }
        }
        // If not present, return Ok(None).
        debug!("No authors field found in CargoToml.");
        Ok(None)
    }
}

impl GetRustEdition for CargoToml {
    type Error = CargoTomlError;

    fn get_rust_edition(&self) -> Result<Option<String>, Self::Error> {
        debug!("Attempting to retrieve `edition` from CargoToml directly.");
        // Typically stored at package.edition
        if let Some(pkg) = self.content().as_table().and_then(|t| t.get("package")) {
            if let Some(pkg_table) = pkg.as_table() {
                if let Some(edition_val) = pkg_table.get("edition").and_then(|v| v.as_str()) {
                    trace!("Found edition='{}' in CargoToml.", edition_val);
                    return Ok(Some(edition_val.to_string()));
                }
            }
        }
        debug!("No edition field found in CargoToml.");
        Ok(None)
    }
}

impl GetLicenseType for CargoToml {
    type Error = CargoTomlError;

    fn get_license_type(&self) -> Result<Option<String>, Self::Error> {
        debug!("Attempting to retrieve `license` from CargoToml directly.");
        // Typically stored at package.license
        if let Some(pkg) = self.content().as_table().and_then(|t| t.get("package")) {
            if let Some(pkg_table) = pkg.as_table() {
                if let Some(lic_val) = pkg_table.get("license").and_then(|v| v.as_str()) {
                    trace!("Found license='{}' in CargoToml.", lic_val);
                    return Ok(Some(lic_val.to_string()));
                }
            }
        }
        debug!("No license field found in CargoToml.");
        Ok(None)
    }
}

impl GetCrateRepositoryLocation for CargoToml {
    type Error = CargoTomlError;

    fn get_crate_repository_location(&self) -> Result<Option<String>, Self::Error> {
        debug!("Attempting to retrieve `repository` from CargoToml directly.");
        // Typically stored at package.repository
        if let Some(pkg) = self.content().as_table().and_then(|t| t.get("package")) {
            if let Some(pkg_table) = pkg.as_table() {
                if let Some(repo_val) = pkg_table.get("repository").and_then(|v| v.as_str()) {
                    trace!("Found repository='{}' in CargoToml.", repo_val);
                    return Ok(Some(repo_val.to_string()));
                }
            }
        }
        debug!("No repository field found in CargoToml.");
        Ok(None)
    }
}
