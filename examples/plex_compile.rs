use std::io::prelude::*;
use std::borrow::Cow;
use std::convert::*;
use std::fs::File;
use std::ptr;

use std::mem::transmute;
use std::ffi;
use std::os::raw::{
    c_uint,
    c_int
};

use endian_codec::EncodeBE;

use otf_fea_rs::{
    GlyphOrder,
    IntoGlyphOrder,
    Tag,
    tag,

    parser,
    compiler,

    compile_model::*
};

use otf_fea_rs::glyph::GlyphRef;


fn plex_glyph_order() -> GlyphOrder {
    let glyphs = [
        (0, ".notdef"),
        (1, "a"),
        (2, "a.alt01"),
        (3, "b"),
        (4, "c"),
        (5, "d"),
        (6, "e"),
        (7, "f"),
        (8, "g"),
        (9, "g.alt01"),
        (10, "g.alt02"),
        (11, "h"),
        (12, "i"),
        (13, "j"),
        (14, "k"),
        (15, "l"),
        (16, "m"),
        (17, "n"),
        (18, "o"),
        (19, "p"),
        (20, "q"),
        (21, "r"),
        (22, "s"),
        (23, "t"),
        (24, "u"),
        (25, "v"),
        (26, "w"),
        (27, "x"),
        (28, "y"),
        (29, "z"),
        (30, "A"),
        (31, "B"),
        (32, "C"),
        (33, "D"),
        (34, "E"),
        (35, "F"),
        (36, "G"),
        (37, "H"),
        (38, "I"),
        (39, "J"),
        (40, "K"),
        (41, "L"),
        (42, "M"),
        (43, "N"),
        (44, "O"),
        (45, "P"),
        (46, "Q"),
        (47, "R"),
        (48, "S"),
        (49, "T"),
        (50, "U"),
        (51, "V"),
        (52, "W"),
        (53, "X"),
        (54, "Y"),
        (55, "Z"),
        (56, "zero"),
        (57, "zero.alt01"),
        (58, "zero.alt02"),
        (59, "one"),
        (60, "two"),
        (61, "three"),
        (62, "four"),
        (63, "five"),
        (64, "six"),
        (65, "seven"),
        (66, "eight"),
        (67, "nine"),
        (68, "ampersand"),
        (69, "at"),
        (70, "hyphen"),
        (71, "softhyphen"),
        (72, "endash"),
        (73, "emdash"),
        (74, "underscore"),
        (75, "period"),
        (76, "ellipsis"),
        (77, "colon"),
        (78, "comma"),
        (79, "semicolon"),
        (80, "questiongreek"),
        (81, "quotesingle"),
        (82, "quotedbl"),
        (83, "quoteleft"),
        (84, "quoteright"),
        (85, "quotedblleft"),
        (86, "quotedblright"),
        (87, "quotesinglbase"),
        (88, "quotedblbase"),
        (89, "guilsinglleft"),
        (90, "guilsinglright"),
        (91, "guillemotleft"),
        (92, "guillemotright"),
        (93, "exclamdown"),
        (94, "exclam"),
        (95, "questiondown"),
        (96, "question"),
        (97, "parenleft"),
        (98, "parenright"),
        (99, "bracketleft"),
        (100, "bracketright"),
        (101, "braceleft"),
        (102, "braceright"),
        (103, "slash"),
        (104, "backslash"),
        (105, "fraction"),
        (106, "divisionslash"),
        (107, "percent"),
        (108, "perthousand"),
        (109, "bar"),
        (110, "brokenbar"),
        (111, "section"),
        (112, "paragraph"),
        (113, "copyright"),
        (114, "registered"),
        (115, "trademark"),
        (116, "ordfeminine"),
        (117, "ordmasculine"),
        (118, "degree"),
        (119, "prime"),
        (120, "primedbl"),
        (121, "asterisk"),
        (122, "dagger"),
        (123, "daggerdbl"),
        (124, "numbersign"),
        (125, "asciicircum"),
        (126, "asciitilde"),
        (127, "plus"),
        (128, "minus"),
        (129, "plusminus"),
        (130, "multiply"),
        (131, "divide"),
        (132, "equal"),
        (133, "approxequal"),
        (134, "notequal"),
        (135, "less"),
        (136, "greater"),
        (137, "lessequal"),
        (138, "greaterequal"),
        (139, "periodcentered"),
        (140, "anoteleia"),
        (141, "bullet"),
        (142, "lozenge"),
        (143, "logicalnot"),
        (144, "radical"),
        (145, "integral"),
        (146, "infinity"),
        (147, "estimated"),
        (148, "litre"),
        (149, "numerosign"),
        (150, "partialdiff"),
        (151, "currency"),
        (152, "cent"),
        (153, "Euro"),
        (154, "florin"),
        (155, "sterling"),
        (156, "dollar"),
        (157, "yen"),
        (158, "baht"),
        (159, "coloncurrency"),
        (160, "lira"),
        (161, "naira"),
        (162, "rupee"),
        (163, "won"),
        (164, "sheqel"),
        (165, "dong"),
        (166, "kip"),
        (167, "tugrik"),
        (168, "peso"),
        (169, "guarani"),
        (170, "hryvnia"),
        (171, "cedi"),
        (172, "tenge"),
        (173, "rupeeindian"),
        (174, "liraturkish"),
        (175, "ruble"),
        (176, "bitcoin"),
        (177, "fi"),
        (178, "fl"),
        (179, "aacute"),
        (180, "abreve"),
        (181, "acaron"),
        (182, "acircumflex"),
        (183, "adieresis"),
        (184, "adotbelow"),
        (185, "agrave"),
        (186, "ahook"),
        (187, "amacron"),
        (188, "aogonek"),
        (189, "aring"),
        (190, "aringacute"),
        (191, "atilde"),
        (192, "abreveacute"),
        (193, "abrevedotbelow"),
        (194, "abrevegrave"),
        (195, "abrevehook"),
        (196, "abrevetilde"),
        (197, "acircumflexacute"),
        (198, "acircumflexdotbelow"),
        (199, "acircumflexgrave"),
        (200, "acircumflexhook"),
        (201, "acircumflextilde"),
        (202, "aacute.alt01"),
        (203, "abreve.alt01"),
        (204, "acaron.alt01"),
        (205, "acircumflex.alt01"),
        (206, "adieresis.alt01"),
        (207, "adotbelow.alt01"),
        (208, "agrave.alt01"),
        (209, "ahook.alt01"),
        (210, "amacron.alt01"),
        (211, "aogonek.alt01"),
        (212, "aring.alt01"),
        (213, "aringacute.alt01"),
        (214, "atilde.alt01"),
        (215, "abreveacute.alt01"),
        (216, "abrevedotbelow.alt01"),
        (217, "abrevegrave.alt01"),
        (218, "abrevehook.alt01"),
        (219, "abrevetilde.alt01"),
        (220, "acircumflexacute.alt01"),
        (221, "acircumflexdotbelow.alt01"),
        (222, "acircumflexgrave.alt01"),
        (223, "acircumflexhook.alt01"),
        (224, "acircumflextilde.alt01"),
        (225, "ae"),
        (226, "aeacute"),
        (227, "cacute"),
        (228, "ccaron"),
        (229, "ccedilla"),
        (230, "ccircumflex"),
        (231, "cdotaccent"),
        (232, "dcaron"),
        (233, "dcroat"),
        (234, "eth"),
        (235, "eacute"),
        (236, "ebreve"),
        (237, "ecaron"),
        (238, "ecircumflex"),
        (239, "edieresis"),
        (240, "edotaccent"),
        (241, "edotbelow"),
        (242, "egrave"),
        (243, "ehook"),
        (244, "emacron"),
        (245, "eogonek"),
        (246, "etilde"),
        (247, "ecircumflexacute"),
        (248, "ecircumflexdotbelow"),
        (249, "ecircumflexgrave"),
        (250, "ecircumflexhook"),
        (251, "ecircumflextilde"),
        (252, "schwa"),
        (253, "gbreve"),
        (254, "gcircumflex"),
        (255, "gcommaaccent"),
        (256, "gdotaccent"),
        (257, "gbreve.alt01"),
        (258, "gcircumflex.alt01"),
        (259, "gcommaaccent.alt01"),
        (260, "gdotaccent.alt01"),
        (261, "hbar"),
        (262, "hcircumflex"),
        (263, "dotlessi"),
        (264, "iacute"),
        (265, "ibreve"),
        (266, "icaron"),
        (267, "icircumflex"),
        (268, "idieresis"),
        (269, "idotbelow"),
        (270, "igrave"),
        (271, "ihook"),
        (272, "imacron"),
        (273, "iogonek"),
        (274, "itilde"),
        (275, "ij"),
        (276, "ijacute"),
        (277, "dotlessj"),
        (278, "jacute"),
        (279, "jcircumflex"),
        (280, "kcommaaccent"),
        (281, "kgreenlandic"),
        (282, "lacute"),
        (283, "lcaron"),
        (284, "lcommaaccent"),
        (285, "ldot"),
        (286, "lslash"),
        (287, "nacute"),
        (288, "ncaron"),
        (289, "ncommaaccent"),
        (290, "ntilde"),
        (291, "napostrophe"),
        (292, "eng"),
        (293, "oacute"),
        (294, "obreve"),
        (295, "ocaron"),
        (296, "ocircumflex"),
        (297, "odieresis"),
        (298, "odotbelow"),
        (299, "ograve"),
        (300, "ohook"),
        (301, "ohungarumlaut"),
        (302, "omacron"),
        (303, "oslash"),
        (304, "oslashacute"),
        (305, "otilde"),
        (306, "ohorn"),
        (307, "ohornacute"),
        (308, "ohorndotbelow"),
        (309, "ohorngrave"),
        (310, "ohornhook"),
        (311, "ohorntilde"),
        (312, "ocircumflexacute"),
        (313, "ocircumflexdotbelow"),
        (314, "ocircumflexgrave"),
        (315, "ocircumflexhook"),
        (316, "ocircumflextilde"),
        (317, "oe"),
        (318, "racute"),
        (319, "rcaron"),
        (320, "rcommaaccent"),
        (321, "sacute"),
        (322, "scaron"),
        (323, "scedilla"),
        (324, "scircumflex"),
        (325, "scommaaccent"),
        (326, "longs"),
        (327, "germandbls"),
        (328, "germandbls.alt01"),
        (329, "tbar"),
        (330, "tcaron"),
        (331, "tcommaaccent"),
        (332, "tcedilla"),
        (333, "thorn"),
        (334, "uacute"),
        (335, "ubreve"),
        (336, "ucaron"),
        (337, "ucircumflex"),
        (338, "udieresis"),
        (339, "udotbelow"),
        (340, "ugrave"),
        (341, "uhook"),
        (342, "uhungarumlaut"),
        (343, "umacron"),
        (344, "uogonek"),
        (345, "uring"),
        (346, "utilde"),
        (347, "uhorn"),
        (348, "uhornacute"),
        (349, "uhorndotbelow"),
        (350, "uhorngrave"),
        (351, "uhornhook"),
        (352, "uhorntilde"),
        (353, "udieresismacron"),
        (354, "udieresisacute"),
        (355, "udieresisgrave"),
        (356, "udieresiscaron"),
        (357, "wacute"),
        (358, "wcircumflex"),
        (359, "wdieresis"),
        (360, "wgrave"),
        (361, "yacute"),
        (362, "ycircumflex"),
        (363, "ydieresis"),
        (364, "ydotbelow"),
        (365, "ygrave"),
        (366, "yhook"),
        (367, "ytilde"),
        (368, "zacute"),
        (369, "zcaron"),
        (370, "zdotaccent"),
        (371, "Aacute"),
        (372, "Abreve"),
        (373, "Acaron"),
        (374, "Acircumflex"),
        (375, "Adieresis"),
        (376, "Adotbelow"),
        (377, "Agrave"),
        (378, "Ahook"),
        (379, "Amacron"),
        (380, "Aogonek"),
        (381, "Aring"),
        (382, "Aringacute"),
        (383, "Atilde"),
        (384, "Abreveacute"),
        (385, "Abrevedotbelow"),
        (386, "Abrevegrave"),
        (387, "Abrevehook"),
        (388, "Abrevetilde"),
        (389, "Acircumflexacute"),
        (390, "Acircumflexdotbelow"),
        (391, "Acircumflexgrave"),
        (392, "Acircumflexhook"),
        (393, "Acircumflextilde"),
        (394, "AE"),
        (395, "AEacute"),
        (396, "Cacute"),
        (397, "Ccaron"),
        (398, "Ccedilla"),
        (399, "Ccircumflex"),
        (400, "Cdotaccent"),
        (401, "Dcaron"),
        (402, "Dcroat"),
        (403, "Eth"),
        (404, "Eacute"),
        (405, "Ebreve"),
        (406, "Ecaron"),
        (407, "Ecircumflex"),
        (408, "Edieresis"),
        (409, "Edotaccent"),
        (410, "Edotbelow"),
        (411, "Egrave"),
        (412, "Ehook"),
        (413, "Emacron"),
        (414, "Eogonek"),
        (415, "Etilde"),
        (416, "Ecircumflexacute"),
        (417, "Ecircumflexdotbelow"),
        (418, "Ecircumflexgrave"),
        (419, "Ecircumflexhook"),
        (420, "Ecircumflextilde"),
        (421, "Schwa"),
        (422, "Gbreve"),
        (423, "Gcircumflex"),
        (424, "Gcommaaccent"),
        (425, "Gdotaccent"),
        (426, "Hbar"),
        (427, "Hcircumflex"),
        (428, "Iacute"),
        (429, "Ibreve"),
        (430, "Icaron"),
        (431, "Icircumflex"),
        (432, "Idieresis"),
        (433, "Idotaccent"),
        (434, "Idotbelow"),
        (435, "Igrave"),
        (436, "Ihook"),
        (437, "Imacron"),
        (438, "Iogonek"),
        (439, "Itilde"),
        (440, "IJ"),
        (441, "IJacute"),
        (442, "Jacute"),
        (443, "Jcircumflex"),
        (444, "Kcommaaccent"),
        (445, "Lacute"),
        (446, "Lcaron"),
        (447, "Lcommaaccent"),
        (448, "Ldot"),
        (449, "Lslash"),
        (450, "Nacute"),
        (451, "Ncaron"),
        (452, "Ncommaaccent"),
        (453, "Ntilde"),
        (454, "Eng"),
        (455, "Oacute"),
        (456, "Obreve"),
        (457, "Ocaron"),
        (458, "Ocircumflex"),
        (459, "Odieresis"),
        (460, "Odotbelow"),
        (461, "Ograve"),
        (462, "Ohook"),
        (463, "Ohungarumlaut"),
        (464, "Omacron"),
        (465, "Oslash"),
        (466, "Oslashacute"),
        (467, "Otilde"),
        (468, "Ohorn"),
        (469, "Ohornacute"),
        (470, "Ohorndotbelow"),
        (471, "Ohorngrave"),
        (472, "Ohornhook"),
        (473, "Ohorntilde"),
        (474, "Ocircumflexacute"),
        (475, "Ocircumflexdotbelow"),
        (476, "Ocircumflexgrave"),
        (477, "Ocircumflexhook"),
        (478, "Ocircumflextilde"),
        (479, "OE"),
        (480, "Racute"),
        (481, "Rcaron"),
        (482, "Rcommaaccent"),
        (483, "Sacute"),
        (484, "Scaron"),
        (485, "Scedilla"),
        (486, "Scircumflex"),
        (487, "Scommaaccent"),
        (488, "Germandbls"),
        (489, "Tbar"),
        (490, "Tcaron"),
        (491, "Tcommaaccent"),
        (492, "Tcedilla"),
        (493, "Thorn"),
        (494, "Uacute"),
        (495, "Ubreve"),
        (496, "Ucaron"),
        (497, "Ucircumflex"),
        (498, "Udieresis"),
        (499, "Udotbelow"),
        (500, "Ugrave"),
        (501, "Uhook"),
        (502, "Uhungarumlaut"),
        (503, "Umacron"),
        (504, "Uogonek"),
        (505, "Uring"),
        (506, "Utilde"),
        (507, "Uhorn"),
        (508, "Uhornacute"),
        (509, "Uhorndotbelow"),
        (510, "Uhorngrave"),
        (511, "Uhornhook"),
        (512, "Uhorntilde"),
        (513, "Udieresismacron"),
        (514, "Udieresisacute"),
        (515, "Udieresisgrave"),
        (516, "Udieresiscaron"),
        (517, "Wacute"),
        (518, "Wcircumflex"),
        (519, "Wdieresis"),
        (520, "Wgrave"),
        (521, "Yacute"),
        (522, "Ycircumflex"),
        (523, "Ydotbelow"),
        (524, "Ydieresis"),
        (525, "Ygrave"),
        (526, "Yhook"),
        (527, "Ytilde"),
        (528, "Zacute"),
        (529, "Zcaron"),
        (530, "Zdotaccent"),
        (531, "mu"),
        (532, "Delta"),
        (533, "product"),
        (534, "summation"),
        (535, "Omega"),
        (536, "alpha"),
        (537, "alpha.alt01"),
        (538, "beta"),
        (539, "gamma"),
        (540, "delta"),
        (541, "epsilon"),
        (542, "zeta"),
        (543, "eta"),
        (544, "theta"),
        (545, "iota"),
        (546, "kappa"),
        (547, "lambda"),
        (548, "uni03BC"),
        (549, "nu"),
        (550, "xi"),
        (551, "omicron"),
        (552, "pi"),
        (553, "rho"),
        (554, "sigma1"),
        (555, "sigma"),
        (556, "tau"),
        (557, "upsilon"),
        (558, "phi"),
        (559, "chi"),
        (560, "psi"),
        (561, "omega"),
        (562, "Alpha"),
        (563, "Beta"),
        (564, "Gamma"),
        (565, "uni0394"),
        (566, "Epsilon"),
        (567, "Zeta"),
        (568, "Eta"),
        (569, "Theta"),
        (570, "Iota"),
        (571, "Kappa"),
        (572, "Lambda"),
        (573, "Mu"),
        (574, "Nu"),
        (575, "Xi"),
        (576, "Omicron"),
        (577, "Pi"),
        (578, "Rho"),
        (579, "Sigma"),
        (580, "Tau"),
        (581, "Upsilon"),
        (582, "Phi"),
        (583, "Chi"),
        (584, "Psi"),
        (585, "uni03A9"),
        (586, "alphatonos"),
        (587, "alphatonos.alt01"),
        (588, "epsilontonos"),
        (589, "etatonos"),
        (590, "iotatonos"),
        (591, "iotadieresis"),
        (592, "iotadieresistonos"),
        (593, "omicrontonos"),
        (594, "upsilontonos"),
        (595, "upsilondieresis"),
        (596, "upsilondieresistonos"),
        (597, "omegatonos"),
        (598, "Alphatonos"),
        (599, "Epsilontonos"),
        (600, "Etatonos"),
        (601, "Iotatonos"),
        (602, "Iotadieresis"),
        (603, "Omicrontonos"),
        (604, "Upsilontonos"),
        (605, "Upsilondieresis"),
        (606, "Omegatonos"),
        (607, "uni0430"),
        (608, "uni0430.alt01"),
        (609, "uni0431"),
        (610, "uni0432"),
        (611, "uni0433"),
        (612, "uni0434"),
        (613, "uni0435"),
        (614, "uni0436"),
        (615, "uni0437"),
        (616, "uni0438"),
        (617, "uni0439"),
        (618, "uni043A"),
        (619, "uni043B"),
        (620, "uni043C"),
        (621, "uni043D"),
        (622, "uni043E"),
        (623, "uni043F"),
        (624, "uni0440"),
        (625, "uni0441"),
        (626, "uni0442"),
        (627, "uni0443"),
        (628, "uni0444"),
        (629, "uni0445"),
        (630, "uni0446"),
        (631, "uni0447"),
        (632, "uni0448"),
        (633, "uni0449"),
        (634, "uni044A"),
        (635, "uni044B"),
        (636, "uni044C"),
        (637, "uni044D"),
        (638, "uni044E"),
        (639, "uni044F"),
        (640, "uni0410"),
        (641, "uni0411"),
        (642, "uni0412"),
        (643, "uni0413"),
        (644, "uni0414"),
        (645, "uni0415"),
        (646, "uni0416"),
        (647, "uni0417"),
        (648, "uni0418"),
        (649, "uni0419"),
        (650, "uni041A"),
        (651, "uni041B"),
        (652, "uni041C"),
        (653, "uni041D"),
        (654, "uni041E"),
        (655, "uni041F"),
        (656, "uni0420"),
        (657, "uni0421"),
        (658, "uni0422"),
        (659, "uni0423"),
        (660, "uni0424"),
        (661, "uni0425"),
        (662, "uni0426"),
        (663, "uni0427"),
        (664, "uni0428"),
        (665, "uni0429"),
        (666, "uni042A"),
        (667, "uni042B"),
        (668, "uni042C"),
        (669, "uni042D"),
        (670, "uni042E"),
        (671, "uni042F"),
        (672, "uni04D3"),
        (673, "uni04D1"),
        (674, "uni04D3.alt01"),
        (675, "uni04D1.alt01"),
        (676, "uni04D5"),
        (677, "uni0453"),
        (678, "uni0491"),
        (679, "uni0493"),
        (680, "uni0495"),
        (681, "uni0450"),
        (682, "uni0451"),
        (683, "uni04D7"),
        (684, "uni0454"),
        (685, "uni04DD"),
        (686, "uni04C2"),
        (687, "uni0497"),
        (688, "uni04DF"),
        (689, "uni0499"),
        (690, "uni04CF"),
        (691, "uni04E5"),
        (692, "uni045D"),
        (693, "uni04E3"),
        (694, "uni045C"),
        (695, "uni049B"),
        (696, "uni049D"),
        (697, "uni04A1"),
        (698, "uni0459"),
        (699, "uni04A3"),
        (700, "uni045A"),
        (701, "uni04A5"),
        (702, "uni04E7"),
        (703, "uni0473"),
        (704, "uni04E9"),
        (705, "uni04AB"),
        (706, "uni04EF"),
        (707, "uni04F1"),
        (708, "uni04F3"),
        (709, "uni045E"),
        (710, "uni04AF"),
        (711, "uni04B1"),
        (712, "uni04B3"),
        (713, "uni04F5"),
        (714, "uni04B7"),
        (715, "uni04B9"),
        (716, "uni04F9"),
        (717, "uni0455"),
        (718, "uni045F"),
        (719, "uni0456"),
        (720, "uni0457"),
        (721, "uni0458"),
        (722, "uni0452"),
        (723, "uni045B"),
        (724, "uni04BB"),
        (725, "uni04D9"),
        (726, "uni04D2"),
        (727, "uni04D0"),
        (728, "uni04D4"),
        (729, "uni0403"),
        (730, "uni0490"),
        (731, "uni0492"),
        (732, "uni0494"),
        (733, "uni0400"),
        (734, "uni0401"),
        (735, "uni04D6"),
        (736, "uni0404"),
        (737, "uni04DC"),
        (738, "uni04C1"),
        (739, "uni0496"),
        (740, "uni04DE"),
        (741, "uni0498"),
        (742, "uni04C0"),
        (743, "uni04E4"),
        (744, "uni040D"),
        (745, "uni04E2"),
        (746, "uni040C"),
        (747, "uni049A"),
        (748, "uni049C"),
        (749, "uni04A0"),
        (750, "uni0409"),
        (751, "uni04A2"),
        (752, "uni040A"),
        (753, "uni04A4"),
        (754, "uni04E6"),
        (755, "uni0472"),
        (756, "uni04E8"),
        (757, "uni04AA"),
        (758, "uni04EE"),
        (759, "uni04F0"),
        (760, "uni04F2"),
        (761, "uni040E"),
        (762, "uni04AE"),
        (763, "uni04B0"),
        (764, "uni04B2"),
        (765, "uni04F4"),
        (766, "uni04B6"),
        (767, "uni04B8"),
        (768, "uni04F8"),
        (769, "uni0405"),
        (770, "uni040F"),
        (771, "uni0406"),
        (772, "uni0407"),
        (773, "uni0408"),
        (774, "uni0402"),
        (775, "uni040B"),
        (776, "uni04BA"),
        (777, "uni04D8"),
        (778, "zerosuperior"),
        (779, "onesuperior"),
        (780, "twosuperior"),
        (781, "threesuperior"),
        (782, "foursuperior"),
        (783, "fivesuperior"),
        (784, "sixsuperior"),
        (785, "sevensuperior"),
        (786, "eightsuperior"),
        (787, "ninesuperior"),
        (788, "zeroinferior"),
        (789, "oneinferior"),
        (790, "twoinferior"),
        (791, "threeinferior"),
        (792, "fourinferior"),
        (793, "fiveinferior"),
        (794, "sixinferior"),
        (795, "seveninferior"),
        (796, "eightinferior"),
        (797, "nineinferior"),
        (798, "onehalf"),
        (799, "uni2153"),
        (800, "uni2154"),
        (801, "onequarter"),
        (802, "threequarters"),
        (803, "uni2155"),
        (804, "uni2156"),
        (805, "uni2157"),
        (806, "uni2158"),
        (807, "uni2159"),
        (808, "uni215A"),
        (809, "uni2150"),
        (810, "uni215B"),
        (811, "uni215C"),
        (812, "uni215D"),
        (813, "uni215E"),
        (814, "uni2151"),
        (815, "checkmark"),
        (816, "crossmark"),
        (817, "arrowleft"),
        (818, "arrowup"),
        (819, "arrowdown"),
        (820, "arrowright"),
        (821, "arrowupleft"),
        (822, "arrowupright"),
        (823, "arrowdownleft"),
        (824, "arrowdownright"),
        (825, "arrowupleftcorner"),
        (826, "arrowdownleftcorner"),
        (827, "arrowleftupcorner"),
        (828, "arrowrightupcorner"),
        (829, "arrowleftdowncorner"),
        (830, "arrowrightdowncorner"),
        (831, "arrowuprightcorner"),
        (832, "arrowdownrightcorner"),
        (833, "arrowleftarrowright"),
        (834, "arrowrightarrowleft"),
        (835, "arrowleftright"),
        (836, "arrowupdown"),
        (837, "arrowdowncounterclockhalf"),
        (838, "arrowdownclockhalf"),
        (839, "arrowhookleft"),
        (840, "arrowhookright"),
        (841, "arrowupleftcounterclock"),
        (842, "arrowuprightclock"),
        (843, "tilde"),
        (844, "tilde.alt01"),
        (845, "macron"),
        (846, "dotaccent"),
        (847, "dieresis"),
        (848, "hungarumlaut"),
        (849, "acute"),
        (850, "grave"),
        (851, "circumflex"),
        (852, "caron"),
        (853, "breve"),
        (854, "breve.cyrl"),
        (855, "ring"),
        (856, "ringacute"),
        (857, "commaturnedtop"),
        (858, "caronslovak"),
        (859, "cedilla"),
        (860, "ogonek"),
        (861, "tonos"),
        (862, "dieresistonos"),
        (863, "tildecomb"),
        (864, "macroncomb"),
        (865, "dotaccentcomb"),
        (866, "dieresiscomb"),
        (867, "hungarumlautcomb"),
        (868, "acutecomb"),
        (869, "gravecomb"),
        (870, "circumflexcomb"),
        (871, "caroncomb"),
        (872, "brevecomb"),
        (873, "ringcomb"),
        (874, "hookcomb"),
        (875, "commaturnedtopcomb"),
        (876, "caronslovakcomb"),
        (877, "horncomb"),
        (878, "cedillacomb"),
        (879, "dotbelowcomb"),
        (880, "commabelowcomb"),
        (881, "ogonekcomb"),
        (882, "breveacute"),
        (883, "brevegrave"),
        (884, "brevehook"),
        (885, "brevetilde"),
        (886, "dieresisacute"),
        (887, "dieresiscaron"),
        (888, "dieresisgrave"),
        (889, "circumflexacute"),
        (890, "circumflexbreve"),
        (891, "circumflexgrave"),
        (892, "circumflexhook"),
        (893, "dieresismacron"),
        (894, "circumflextilde"),
        (895, "tilde.case"),
        (896, "tilde.alt01.case"),
        (897, "macron.case"),
        (898, "dotaccent.case"),
        (899, "dieresis.case"),
        (900, "hungarumlaut.case"),
        (901, "acute.case"),
        (902, "grave.case"),
        (903, "circumflex.case"),
        (904, "caron.case"),
        (905, "breve.case"),
        (906, "breve.cyrl_case"),
        (907, "ring.case"),
        (908, "ringacute.case"),
        (909, "tonos.case"),
        (910, "hookcomb.case"),
        (911, "breveacute.case"),
        (912, "brevegrave.case"),
        (913, "brevehook.case"),
        (914, "brevetilde.case"),
        (915, "dieresisacute.case"),
        (916, "dieresiscaron.case"),
        (917, "dieresisgrave.case"),
        (918, "circumflexacute.case"),
        (919, "circumflexbreve.case"),
        (920, "circumflexgrave.case"),
        (921, "circumflexhook.case"),
        (922, "dieresismacron.case"),
        (923, "circumflextilde.case"),
        (924, "space"),
        (925, "nbspace"),
        (926, "fcclogo"),
        (927, "celogo"),
    ];

    glyphs
        .iter()
        .map(|(id, name)| (*id as u16, GlyphRef::from_name(name)))
        .collect_into_glyph_order()
        .unwrap()
}

pub fn recompile(in_fea_path: &str, in_ttf_path: &str) {
    println!();
    println!("loading `{}`...", &in_ttf_path);
    let mut ttf_buf = Vec::new();
    let mut tables = {
        let mut f = File::open(&in_ttf_path).unwrap();
        f.read_to_end(&mut ttf_buf).unwrap();
        EncodedTables::from_ttf_file(&ttf_buf).unwrap()
    };
    println!("    loaded successfully!");
    println!();

    let glyph_order = plex_glyph_order();

    println!();
    println!("parsing `{}`...", &in_fea_path);
    let fea = File::open(&in_fea_path).unwrap();
    let parsed = parser::parse_file(fea).unwrap();
    println!("    parsed successfully!");
    println!();

    println!("compiling...");
    compiler::compile(glyph_order, &parsed)
        .unwrap()
        .merge_encoded_tables(&mut tables)
        .unwrap();
    println!("    compiled successfully!");
    println!();
}

pub struct State {
    #[allow(dead_code)]
    ttf_buf: Vec<u8>,
    tables: EncodedTables<'static>
}

impl State {
    fn new_from_ttf(ttf_path: &str) -> Result<Self, ()> {
        let mut ttf_buf = Vec::new();

        let mut f = File::open(&ttf_path)
            .map_err(|e|
                println!("File::open() failed: {}", e))?;

        f.read_to_end(&mut ttf_buf)
            .map_err(|e|
                println!("read_to_end() failed: {}", e))?;

        let tables = unsafe {
            // unsafe/transmute to cast lifetime to 'static

            EncodedTables::from_ttf_file(transmute(ttf_buf.as_slice()))
                .map_err(|e|
                    println!("EncodedTables::from_ttf_file() failed: {}", e))
        }?;

        Ok(Self {
            ttf_buf,
            tables
        })
    }

    #[inline]
    fn lookup_table(&self, tag: u32) -> Result<&[u8], ()> {
        let tag = Tag::try_from(tag)
            .map_err(|_| ())?;

        match self.tables.get_table(tag) {
            Some(t) => Ok(&t.bytes),
            _ => Err(())
        }
    }

    #[inline]
    fn merge_features(&mut self, fea_path: &str) -> Result<(), ()> {
        let glyph_order = plex_glyph_order();

        println!();
        println!("parsing `{}`...", &fea_path);
        let fea = File::open(&fea_path)
            .map_err(|e|
                println!("File::open() failed: {}", e))?;

        let parsed = parser::parse_file(fea)
            .map_err(|_| println!("parse_file() failed"))?;

        println!("    parsed successfully!");
        println!();

        println!("compiling...");
        compiler::compile(glyph_order, &parsed)
            .map_err(|e| println!("compile() failed: {}", e))?
            .merge_encoded_tables(&mut self.tables)
            .map_err(|e| println!("merge_encoded_tables() failed: {}", e))?;
        println!("    compiled successfully!");
        println!();

        Ok(())
    }

    #[inline]
    fn clobber_bits<'a>(&'a mut self, idx: c_uint, delta: i16) {
        let gpos = {
            let tables = unsafe { transmute::<_, &mut EncodedTables<'a>>(&mut self.tables) };

            match tables.get_table_mut(tag!(G,P,O,S)) {
                Some(t) => t,
                None => return
            }
        };

        let pos = match idx {
            0 => (468, 0),
            1 => (468, 1),
            2 => (469, 0),
            3 => (470, 0),
            _ => return
        };

        let map = match gpos.source_map.get(&pos.0) {
            Some(m) => match m.values().nth(pos.1) {
                Some(CompiledEntry::I16(o)) => *o,
                None => {
                    println!("what");
                    return
                }
            },

            None => {
                println!("no source map for loc");
                return
            }
        };

        let slice = match gpos.bytes {
            Cow::Borrowed(_) => {
                println!("not clobbering borrowed bytes");
                return
            },

            Cow::Owned(ref mut bytes) => {
                &mut bytes[map..map+2]
            }
        };

        let cv = {
            let mut a = [0u8; 2];
            a.copy_from_slice(slice);
            i16::from_be_bytes(a)
        };

        let cv = cv.saturating_add(delta);
        println!("{}", cv);

        cv.encode_as_be_bytes(slice);
    }
}

#[no_mangle]
pub unsafe extern "C" fn frs_new(ttf_path: *const i8) -> *mut State {
    if ttf_path == ptr::null_mut() {
        return ptr::null_mut();
    }

    let ttf_path_str = match ffi::CStr::from_ptr(ttf_path).to_str() {
        Ok(s) => s,
        Err(e) => {
            println!("`ttf_path` isn't valid UTF-8: {}", e);
            return ptr::null_mut();
        }
    };

    match State::new_from_ttf(ttf_path_str) {
        Ok(state) => Box::into_raw(Box::new(state)),
        Err(_) => ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn frs_destroy(state: *mut State) {
    if state == ptr::null_mut() {
        return;
    }

    drop(Box::from_raw(state));
}

#[no_mangle]
pub extern "C" fn frs_lookup_table(state: *const State, tag: u32, size: *mut c_uint) -> *const u8 {
    let state = unsafe { match state.as_ref() {
        Some(s) => s,
        None => return ptr::null()
    }};

    match state.lookup_table(tag) {
        Ok(buf) => if let Ok(sz) = buf.len().try_into() {
            unsafe {
                *size = sz;
            }

            return buf.as_ptr();
        },

        _ => (),
    }

    unsafe {
        *size = 0;
    }

    ptr::null()
}

#[no_mangle]
pub extern "C" fn frs_merge_features(state: *mut State, fea_path: *const i8) -> c_int {
    let state = unsafe { match state.as_mut() {
        Some(s) => s,
        None => return -1
    }};

    let fea_path = match unsafe { ffi::CStr::from_ptr(fea_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            println!("`fea_path` isn't valid UTF-8: {}", e);
            return -1;
        }
    };

    state.merge_features(fea_path)
        .map(|_| 0)
        .unwrap_or(1)
}

#[no_mangle]
pub extern "C" fn frs_clobber_bits(state: *mut State, idx: c_uint, delta: i16) {
    let state = unsafe { match state.as_mut() {
        Some(s) => s,
        None => return
    }};

    state.clobber_bits(idx, delta);
}
