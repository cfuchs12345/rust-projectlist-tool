const pageForObject = $('meta[name=objecttype]').attr('content');

const idPlaceholder = "${id}";

const createorupdatePrefix = "createorupdate-" + pageForObject + "-button";
const createorupdateFormId = "createorupdate-" + pageForObject + "-form";
const editPrefix = "edit-" + pageForObject + "-button-";
const editUrl = "/edit" + pageForObject + "/" + idPlaceholder;
const deletePrefix = "delete-" + pageForObject + "-button-";
const deleteUrl = "/delete" + pageForObject + "/" + idPlaceholder;

const navPrefix = "nav-";

function initListeners() {    
    $("[id^=" + editPrefix + "]").on("click", function (e) { editObject(e); });
    $("[id^=" + deletePrefix + "]").on("click", function (e) { deleteObject(e); });
    $("[id^=" + createorupdatePrefix + "]").on("click", function (e) { saveObject(); });
    $("[id^=nav-]").on("click", function (e) { showContent(e); });
    $("[id=generate-de_pdf-button]").on("click", function (e) { generatePDF("de"); });
    $("[id=generate-en_pdf-button]").on("click", function (e) { generatePDF("en"); });
    $("[id=send-projectlist-button]").on("click", function (e) { pushprojectlist(); });
}

$(function() {
    initListeners();
});


$(window).on('load', function () {
    handleActiveNavElement();
});

function generatePDF(language) {
    location.replace('/generate_pdf_projectlist?language=' + language);
}

function pushprojectlist() {
    location.replace('/pushprojectlist');
}

function editObject(e) {
    let objectId = e.target.id.replace(editPrefix, "");
    let url = editUrl.replace(idPlaceholder, objectId);

    location.replace(url);
}

function deleteObject(e) {
    let objectId = e.target.id.replace(deletePrefix, "");
    let url = deleteUrl.replace(idPlaceholder, objectId);

    if (confirm("Are you sure you want to delete this entry?")) {
        $.get(url, function (ret_data) {
            location.reload();
        });
    }
}

function saveObject(event) {
    let form = $("[id=" + createorupdateFormId + "]");
    if (form[0].checkValidity()) {

        // workaround - Actix / Serde from Rust doesn't seem to be able to handle mutli selections in option elements
        // even with [] notation, it doesn't work with multiple values
        // we copy the multi selection select value to a hidden field with the same name but without the brackets
        $("#" + form.attr("id") + " select").each(
            function (index) {
                let input = $(this);
                let multiSelectname = input.attr("name");
                if (multiSelectname.endsWith("[]")) {
                    let hiddenName = multiSelectname.replace("[]", "");
                    let hiddenField = $('input[name="' + hiddenName + '"]');
                    if (hiddenField != undefined) {
                        hiddenField.val(input.val());
                    }
                }
            }
        );

        form.submit();
    }
    else {
        event.preventDefault();
        event.stopPropagation();

        form.addClass('was-validated')
    }
}

function showContent(e) {
    let navId = e.target.id.replace(navPrefix, "");
    let url = "/list" + navId;

    location.assign(url);
}

function handleActiveNavElement(e) {
    let navId = pageForObject;

    $("[id^=nav-]").removeClass("active");
    $("[id^=nav-" + navId + "]").addClass("active");
}
